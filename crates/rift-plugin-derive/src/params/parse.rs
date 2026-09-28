//! Parsing of a `#[derive(Params)]` struct into a small IR.
//!
//! Every field must carry exactly one of `#[param(...)]` (a leaf parameter) or
//! `#[nested]` / `#[nested(module = "...")]` (a struct, or an array of structs,
//! that itself derives `Params`).

use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Generics, Ident, LitStr, Meta, Type,
    meta::ParseNestedMeta,
};

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
    pub name: Option<LitStr>,
    pub id: Option<LitStr>,
    pub unit: Option<LitStr>,
    pub default: Option<Expr>,
    pub range: Option<(Expr, Expr)>,
    pub mapping: Option<Expr>,
    pub flags: Option<Expr>,
}

/// A `#[nested]` field: either a struct or an array of structs.
#[derive(Debug)]
pub(crate) struct Nested {
    pub ident: Ident,
    /// The element type, i.e. the struct behind the field (unwrapped for arrays).
    pub element_ty: Type,
    /// The array length, for `[T; N]` fields.
    pub array_len: Option<Expr>,
    pub module: Option<LitStr>,
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

    let mut param = None;
    let mut nested = None;
    for attr in &field.attrs {
        if attr.path().is_ident("param") {
            if param.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate `#[param]` attribute",
                ));
            }
            param = Some(attr);
        } else if attr.path().is_ident("nested") {
            if nested.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate `#[nested]` attribute",
                ));
            }
            nested = Some(attr);
        }
    }

    match (param, nested) {
        (Some(_), Some(_)) => Err(syn::Error::new_spanned(
            &field.ty,
            "a field cannot be both `#[param]` and `#[nested]`",
        )),
        (Some(attr), None) => parse_leaf(field, attr),
        (None, Some(attr)) => parse_nested(field, attr),
        (None, None) => Err(syn::Error::new(
            ident.span(),
            "every field must be marked `#[param]` or `#[nested]`",
        )),
    }
}

fn parse_leaf(field: &syn::Field, attr: &Attribute) -> syn::Result<Field> {
    let ident = field.ident.clone().expect("named field");
    let ty = field.ty.clone();
    let kind = param_kind(&ty)?;

    let mut leaf = Leaf {
        ident,
        ty,
        kind,
        name: None,
        id: None,
        unit: None,
        default: None,
        range: None,
        mapping: None,
        flags: None,
    };

    match &attr.meta {
        Meta::List(_) => attr.parse_nested_meta(|meta| parse_leaf_entry(&mut leaf, meta))?,
        Meta::Path(_) => {}
        Meta::NameValue(_) => {
            return Err(syn::Error::new_spanned(
                attr,
                "expected `#[param(...)]`, not `#[param = ...]`",
            ));
        }
    }

    validate_leaf(&leaf)?;
    Ok(Field::Leaf(leaf))
}

fn parse_leaf_entry(leaf: &mut Leaf, meta: ParseNestedMeta) -> syn::Result<()> {
    if meta.path.is_ident("name") {
        leaf.name = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("id") {
        leaf.id = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("unit") {
        leaf.unit = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("default") {
        leaf.default = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("range") {
        let expr: Expr = meta.value()?.parse()?;
        leaf.range = Some(parse_range(&expr)?);
    } else if meta.path.is_ident("mapping") {
        leaf.mapping = Some(meta.value()?.parse()?);
    } else if meta.path.is_ident("flags") {
        leaf.flags = Some(meta.value()?.parse()?);
    } else {
        return Err(meta.error(
            "unknown `#[param]` key; expected one of `name`, `id`, `unit`, `default`, \
             `range`, `mapping`, `flags`",
        ));
    }
    Ok(())
}

fn parse_range(expr: &Expr) -> syn::Result<(Expr, Expr)> {
    let Expr::Range(range) = expr else {
        return Err(syn::Error::new_spanned(
            expr,
            "`range` expects a range like `0.0..1.0`",
        ));
    };

    match (&range.start, &range.end) {
        (Some(start), Some(end)) => Ok(((**start).clone(), (**end).clone())),
        _ => Err(syn::Error::new_spanned(
            expr,
            "`range` needs both bounds, e.g. `-60.0..6.0`",
        )),
    }
}

fn validate_leaf(leaf: &Leaf) -> syn::Result<()> {
    if leaf.range.is_some() && matches!(leaf.kind, ParamKind::Bool | ParamKind::Enum) {
        return Err(syn::Error::new_spanned(
            &leaf.ty,
            "`range` is only supported for `FloatParam` and `IntParam`",
        ));
    }

    if leaf.mapping.is_some() && leaf.kind != ParamKind::Float {
        return Err(syn::Error::new_spanned(
            &leaf.ty,
            "`mapping` is only supported for float parameters",
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

fn parse_nested(field: &syn::Field, attr: &Attribute) -> syn::Result<Field> {
    let ident = field.ident.clone().expect("named field");

    let mut module = None;
    match &attr.meta {
        Meta::List(_) => attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("module") {
                module = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unknown `#[nested]` key; expected `module`"))
            }
        })?,
        Meta::Path(_) => {}
        Meta::NameValue(_) => {
            return Err(syn::Error::new_spanned(
                attr,
                "expected `#[nested]` or `#[nested(module = \"...\")]`",
            ));
        }
    }

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
