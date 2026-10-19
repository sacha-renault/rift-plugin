//! Parsing of a `#[derive(Params)]` struct into a small IR.
//!
//! Field attributes are parsed with `darling` (as in the enum derive). The
//! struct layout, the leaf/nested discrimination and the [`ParamKind`] deduced
//! from the field type stay hand-written.
//!
//! Every field must carry exactly one of `#[param(...)]` (a host-visible leaf
//! parameter), `#[persist(...)]` (a host-invisible persistent value) or
//! `#[nested]` / `#[nested(module = "...")]` (a struct, or an array of structs,
//! that itself derives `Params`).

use std::collections::HashSet;

use darling::FromField;
use syn::{Data, DeriveInput, Expr, Fields, Generics, Ident, Path, Type};

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
    tags: Option<Expr>,
}

#[derive(FromField)]
#[darling(attributes(persist))]
struct PersistArgs {
    id: Option<String>,
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
    Param(LeafParam),
    Persist(LeafPersist),
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
pub(crate) struct LeafParam {
    pub ident: Ident,
    pub ty: Type,
    pub kind: ParamKind,
    pub name: Option<String>,
    pub id: Option<String>,
    pub unit: Option<String>,
    pub default: Option<Expr>,
    pub range: Option<RangeSpec>,
    pub flags: Option<Expr>,
    pub tags: Option<Expr>,
}

impl LeafParam {
    pub fn resolve_string_id(&self) -> String {
        self.id.clone().unwrap_or(self.ident.to_string())
    }
}

/// A `#[persist(...)]` field: a host-invisible value saved in the `persist`
/// section of the plugin state.
#[derive(Debug)]
pub(crate) struct LeafPersist {
    pub ident: Ident,
    pub ty: Type,
    pub id: Option<String>,
}

impl LeafPersist {
    pub fn resolve_string_id(&self) -> String {
        self.id.clone().unwrap_or(self.ident.to_string())
    }
}

/// A parsed `range = ...` declaration: the constructor to call (e.g. `linear`,
/// `skew`, `exp`) and its arguments, forwarded verbatim to the kind's range type.
///
/// Arguments are *not* validated here: the range constructors are `const fn`
/// with their own asserts, and the generated call is wrapped in a `const` block,
/// so a bad argument is a compile error.
#[derive(Debug)]
pub(crate) struct RangeSpec {
    pub ctor: Path,
    pub args: Vec<Expr>,
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

impl Nested {
    pub fn resolve_string_id(&self) -> String {
        self.module.clone().unwrap_or(self.ident.to_string())
    }
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

        validate_resolved_ids(&fields)?;

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
    let has_persist = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("persist"));

    match (has_param, has_persist, has_nested) {
        (true, false, false) => parse_param_leaf(field),
        (false, true, false) => parse_persist_leaf(field),
        (false, false, true) => parse_nested(field),
        (false, false, false) => Err(syn::Error::new(
            ident.span(),
            "every field must be marked `#[param]`, `#[persist]` or `#[nested]`",
        )),
        _ => Err(syn::Error::new(
            ident.span(),
            "a field can have only one of `#[param]`, `#[persist]` and `#[nested]`",
        )),
    }
}

fn parse_persist_leaf(field: &syn::Field) -> syn::Result<Field> {
    let ident = field.ident.clone().expect("named field");
    let ty = field.ty.clone();

    let PersistArgs { id } = PersistArgs::from_field(field)?;
    let leaf = LeafPersist { ident, ty, id };

    Ok(Field::Persist(leaf))
}

fn parse_param_leaf(field: &syn::Field) -> syn::Result<Field> {
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
        tags,
    } = ParamArgs::from_field(field)?;

    let range = range.as_ref().map(parse_range).transpose()?;

    let leaf = LeafParam {
        ident,
        ty,
        kind,
        name,
        id,
        unit,
        default,
        range,
        flags,
        tags,
    };

    validate_leaf(&leaf)?;
    Ok(Field::Param(leaf))
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

/// The error for a `range` that is not a constructor call.
const RANGE_EXPECTED: &str = "`range` expects a call such as `linear(min, max)`, \
     `skew(min, max, factor)` or `exp(min, max, factor)`";

/// Parse `range = <ctor>(<args...>)`.
///
/// The constructor name and arguments are forwarded as-is. Their validity (a
/// curve only makes sense on a float, `factor > 0`, …) is enforced by the range
/// constructors at compile time; the macro only rejects `range` on a kind that
/// has no range type at all (see [`validate_leaf`]).
fn parse_range(expr: &Expr) -> syn::Result<RangeSpec> {
    let Expr::Call(call) = expr else {
        return Err(syn::Error::new_spanned(expr, RANGE_EXPECTED));
    };
    let Expr::Path(func) = &*call.func else {
        return Err(syn::Error::new_spanned(expr, RANGE_EXPECTED));
    };

    Ok(RangeSpec {
        ctor: func.path.clone(),
        args: call.args.iter().cloned().collect(),
    })
}

fn validate_leaf(leaf: &LeafParam) -> syn::Result<()> {
    // `range` on a kind without a range type has nowhere to land, so it is the
    // one `range` error the macro still reports itself.
    if leaf.range.is_some() && matches!(leaf.kind, ParamKind::Bool | ParamKind::Enum) {
        return Err(syn::Error::new_spanned(
            &leaf.ty,
            "`range` is only supported for `FloatParam` and `IntParam`",
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

/// The characters allowed in a parameter id or a nested module segment.
///
/// Character allowed are `is_alphanumeric` & `_`
fn is_id_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Validates the resolved id of every field: `#[param(id = "...")]` for a leaf,
/// `#[nested(module = "...")]` for a module, each defaulting to the field name.
///
/// Within one struct, a resolved id must be non-empty, made only of
/// [`is_id_char`] characters, and unique.
fn validate_resolved_ids(fields: &[Field]) -> syn::Result<()> {
    let mut seen = HashSet::new();

    for field in fields.iter() {
        let (resolved_id, ident) = match field {
            Field::Param(leaf) => (leaf.resolve_string_id(), &leaf.ident),
            Field::Persist(leaf) => (leaf.resolve_string_id(), &leaf.ident),
            Field::Nested(nested) => (nested.resolve_string_id(), &nested.ident),
        };

        if resolved_id.is_empty() {
            return Err(syn::Error::new_spanned(
                ident,
                "a param id or module must not be empty",
            ));
        }

        if let Some(bad) = resolved_id.chars().find(|c| !is_id_char(*c)) {
            return Err(syn::Error::new_spanned(
                ident,
                format!(
                    "`{bad}` is not allowed in a param id or module; use only letters, digits and `_`"
                ),
            ));
        }

        if seen.contains(&resolved_id) {
            return Err(syn::Error::new_spanned(
                ident,
                format!("Duplicate id: `{resolved_id}`"),
            ));
        }
        seen.insert(resolved_id);
    }

    Ok(())
}
