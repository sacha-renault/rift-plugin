//! Parsing of a `#[derive(Params)]` struct into a small IR.
//!
//! Field attributes are parsed with `darling` (as in the enum derive). The
//! struct layout, the leaf/nested discrimination and the [`ParamKind`] deduced
//! from the field type stay hand-written.
//!
//! Every field must carry exactly one of `#[param(...)]` (a leaf parameter) or
//! `#[nested]` / `#[nested(module = "...")]` (a struct, or an array of structs,
//! that itself derives `Params`).

use darling::FromField;
use syn::{Data, DeriveInput, Expr, Fields, Generics, Ident, Type};

/// Receiver for `#[param(...)]` on a leaf field.
#[derive(FromField)]
#[darling(attributes(param))]
struct ParamArgs {
    name: Option<String>,
    id: Option<String>,
    unit: Option<String>,
    default: Option<Expr>,
    range: Option<Expr>,
    flags: Option<Expr>,
}

/// Receiver for `#[nested(...)]` on a nested field.
#[derive(FromField)]
#[darling(attributes(nested))]
struct NestedArgs {
    module: Option<String>,
}

/// A parsed `#[derive(Params)]` struct.
#[derive(Debug)]
pub(crate) struct Params {
    pub ident: Ident,
    pub generics: Generics,
    pub fields: Vec<Field>,
}

/// A field of the struct.
// The IR is built once per macro invocation, so the size difference between the
// two variants does not matter.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum Field {
    Leaf(Leaf),
    Nested(Nested),
}

/// The numeric flavour of a leaf parameter, used to coerce literals and to
/// validate which attributes apply.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ParamKind {
    Float,
    Int,
    Bool,
    Enum,
}

/// A `#[param(...)]` field.
#[derive(Debug)]
pub(crate) struct Leaf {
    pub ident: Ident,
    pub ty: Type,
    pub kind: ParamKind,
    pub name: Option<String>,
    pub id: Option<String>,
    pub unit: Option<String>,
    pub default: Option<Expr>,
    pub range: Option<(Expr, Expr)>,
    pub scale: Option<ScaleArg>,
    pub flags: Option<Expr>,
}

/// The curve encoded in a `range` declaration for a float param.
///
/// `range = linear(...)` carries no curve; `range = skew(min, max, factor)` is a
/// power curve whose exponent is stored as written and coerced to `f32` at
/// expansion.
#[derive(Debug)]
pub(crate) enum ScaleArg {
    /// The `factor` of `range = skew(min, max, factor)`.
    Skew(Expr),
}

/// A `#[nested]` field: either a struct or an array of structs.
#[derive(Debug)]
pub(crate) struct Nested {
    pub ident: Ident,
    /// The element type, i.e. the struct behind the field (unwrapped for arrays).
    pub element_ty: Type,
    /// The array length, for `[T; N]` fields.
    pub array_len: Option<Expr>,
    pub module: Option<String>,
}

impl Params {
    pub(crate) fn from_derive_input(input: &DeriveInput) -> syn::Result<Self> {
        let Data::Struct(data) = &input.data else {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "`Params` can only be derived for structs",
            ));
        };

        let Fields::Named(named) = &data.fields else {
            return Err(syn::Error::new_spanned(
                &data.fields,
                "`Params` requires named fields",
            ));
        };

        let fields = named
            .named
            .iter()
            .map(parse_field)
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(Self {
            ident: input.ident.clone(),
            generics: input.generics.clone(),
            fields,
        })
    }
}

fn parse_field(field: &syn::Field) -> syn::Result<Field> {
    let ident = field
        .ident
        .clone()
        .expect("named fields always have an identifier");

    let has_param = field.attrs.iter().any(|attr| attr.path().is_ident("param"));
    let has_nested = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("nested"));

    match (has_param, has_nested) {
        (true, true) => Err(syn::Error::new_spanned(
            &field.ty,
            "a field cannot be both `#[param]` and `#[nested]`",
        )),
        (true, false) => parse_leaf(field),
        (false, true) => parse_nested(field),
        (false, false) => Err(syn::Error::new(
            ident.span(),
            "every field must be marked `#[param]` or `#[nested]`",
        )),
    }
}

fn parse_leaf(field: &syn::Field) -> syn::Result<Field> {
    let ident = field.ident.clone().expect("named field");
    let ty = field.ty.clone();
    let kind = param_kind(&ty)?;

    let ParamArgs {
        name,
        id,
        unit,
        default,
        range,
        flags,
    } = ParamArgs::from_field(field)?;

    let (range, scale) = match range.as_ref() {
        Some(expr) => {
            let (bounds, scale) = parse_range(expr)?;
            (Some(bounds), scale)
        }
        None => (None, None),
    };

    let leaf = Leaf {
        ident,
        ty,
        kind,
        name,
        id,
        unit,
        default,
        range,
        scale,
        flags,
    };

    validate_leaf(&leaf)?;
    Ok(Field::Leaf(leaf))
}

fn parse_nested(field: &syn::Field) -> syn::Result<Field> {
    let ident = field.ident.clone().expect("named field");
    let NestedArgs { module } = NestedArgs::from_field(field)?;

    let (element_ty, array_len) = match &field.ty {
        Type::Array(array) => ((*array.elem).clone(), Some(array.len.clone())),
        other => (other.clone(), None),
    };

    if !matches!(element_ty, Type::Path(_)) {
        return Err(syn::Error::new_spanned(
            &field.ty,
            "`#[nested]` requires a struct type or an array of struct types",
        ));
    }

    Ok(Field::Nested(Nested {
        ident,
        element_ty,
        array_len,
        module,
    }))
}

/// One accepted `range = ...` form.
///
/// Adding a curve means adding one entry to [`RANGE_VARIANTS`]: the name match,
/// the arity check and the "expected" error text are all derived from the table.
struct RangeVariant {
    /// The DSL keyword, e.g. `skew`.
    name: &'static str,
    /// Argument names *after* the implicit `min, max`; used for arity and errors.
    extra_args: &'static [&'static str],
    /// Builds the curve from `extra_args`, or `None` for a linear range.
    build: fn(&[Expr]) -> Option<ScaleArg>,
}

/// Every accepted `range` form. `min` and `max` are implicit and always the
/// first two arguments, so they are not listed in `extra_args`.
///
/// To add a form, follow the "Adding a new curve" procedure in the `scale`
/// module of `rift-plugin-params`.
const RANGE_VARIANTS: &[RangeVariant] = &[
    RangeVariant {
        name: "linear",
        extra_args: &[],
        build: |_| None,
    },
    RangeVariant {
        name: "skew",
        extra_args: &["factor"],
        build: |extra| Some(ScaleArg::Skew(extra[0].clone())),
    },
];

/// A `range = ...` value, parsed against [`RANGE_VARIANTS`].
///
/// `linear(min, max)` yields the plain bounds with no curve; `skew(min, max,
/// factor)` additionally carries a power curve. Float params may use any form;
/// int, bool and enum params may only use `linear` (see [`validate_leaf`]).
fn parse_range(expr: &Expr) -> syn::Result<((Expr, Expr), Option<ScaleArg>)> {
    let Expr::Call(call) = expr else {
        return Err(syn::Error::new_spanned(expr, expected_range_message()));
    };
    let Expr::Path(func) = &*call.func else {
        return Err(syn::Error::new_spanned(expr, expected_range_message()));
    };

    let Some(variant) = RANGE_VARIANTS
        .iter()
        .find(|variant| func.path.is_ident(variant.name))
    else {
        return Err(syn::Error::new_spanned(func, expected_range_message()));
    };

    // `min` and `max` are always present; the rest is `extra_args`.
    if call.args.len() != 2 + variant.extra_args.len() {
        return Err(syn::Error::new_spanned(
            &call.args,
            expected_range_message(),
        ));
    }

    let min = call.args[0].clone();
    let max = call.args[1].clone();
    let extra: Vec<Expr> = call.args.iter().skip(2).cloned().collect();

    Ok(((min, max), (variant.build)(&extra)))
}

/// The shared "expected" error text, generated from [`RANGE_VARIANTS`] so it can
/// never drift from the accepted forms.
fn expected_range_message() -> String {
    let forms: Vec<String> = RANGE_VARIANTS
        .iter()
        .map(|variant| {
            let args = std::iter::once("min")
                .chain(std::iter::once("max"))
                .chain(variant.extra_args.iter().copied())
                .collect::<Vec<_>>()
                .join(", ");
            format!("`{}({args})`", variant.name)
        })
        .collect();

    format!("`range` expects {}", forms.join(" or "))
}

fn validate_leaf(leaf: &Leaf) -> syn::Result<()> {
    if leaf.range.is_some() && matches!(leaf.kind, ParamKind::Bool | ParamKind::Enum) {
        return Err(syn::Error::new_spanned(
            &leaf.ty,
            "`range` is only supported for `FloatParam` and `IntParam`",
        ));
    }

    if leaf.scale.is_some() && leaf.kind != ParamKind::Float {
        return Err(syn::Error::new_spanned(
            &leaf.ty,
            "a `skew` range is only supported for float params \
             (int, bool and enum params are always linear)",
        ));
    }

    Ok(())
}

fn param_kind(ty: &Type) -> syn::Result<ParamKind> {
    if let Type::Array(_) = ty {
        return Err(syn::Error::new_spanned(
            ty,
            "arrays of leaf params are not supported; only arrays of `#[nested]` structs are",
        ));
    }

    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(
            ty,
            "unsupported parameter type; expected a `FloatParam`, `IntParam`, `BoolParam` \
             or `EnumParam<_>` field",
        ));
    };

    let Some(segment) = type_path.path.segments.last() else {
        return Err(syn::Error::new_spanned(ty, "unsupported parameter type"));
    };

    Ok(match segment.ident.to_string().as_str() {
        "FloatParam" | "SharedFloatParam" => ParamKind::Float,
        "IntParam" => ParamKind::Int,
        "BoolParam" => ParamKind::Bool,
        "EnumParam" => ParamKind::Enum,
        other => {
            return Err(syn::Error::new_spanned(
                ty,
                format!(
                    "unsupported parameter type `{other}`; expected `FloatParam`, \
                     `SharedFloatParam`, `IntParam`, `BoolParam` or `EnumParam<_>`"
                ),
            ));
        }
    })
}
