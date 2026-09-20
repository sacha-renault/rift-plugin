//! Parsing of the `params!` token stream into an AST, plus the collection pass
//! that flattens it into named structs.

use proc_macro2::Span;
use std::collections::HashSet;
use syn::{
    Expr, ExprPath, GenericArgument, Ident, PathArguments, Result, Token, Type, braced,
    parse::{Parse, ParseBuffer, ParseStream},
    token::Brace,
};

/// Parsed content of `params! { ... }`.
///
/// The grammar is a list of struct definitions:
///
/// ```ignore
/// params! {
///     struct Name {
///         param field: ParamType { key: value, .. },  // leaf (kind keyword)
///         group: { param field: ParamType { .. } },     // anonymous group
///         other: OtherStructName,                      // reference
///         many: Array<8, OtherStructName>,             // array of a named struct
///         anon: Array<8> { param field: ParamType },   // array of an anonymous struct
///     }
///
///     params {
///         left: Name,
///         voices: Array<8, Name>,
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Params {
    /// All the top level `struct` definitions, in source order.
    pub structs: Vec<Struct>,
    /// The optional `params { .. }` block defining the root `Parameters` struct.
    pub root: Option<Root>,
}

/// The `params { left: ChannelParam, .. }` root block.
///
/// This is the only place that knows the full tree, so it is the sole owner of
/// `Parameters::create()`, the ids and the module paths.
#[derive(Debug)]
pub struct Root {
    pub fields: Vec<Field>,
}

/// A named struct definition, e.g. `struct OscillatorParam { .. }`.
#[derive(Debug)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
}

/// A single field of a struct.
#[derive(Debug)]
pub struct Field {
    pub name: Ident,
    pub value: FieldValue,
}

/// The value of a field, which tells us whether we hit a leaf or a group.
#[derive(Debug)]
pub enum FieldValue {
    /// A leaf introduced by a kind keyword, e.g. `param gain: FloatParam { .. }`.
    ///
    /// Other kinds (`meter`, ...) will be added here later.
    Leaf(Leaf),

    /// A reference to another named struct: `left: ChannelParam`.
    Reference(Type),

    /// An anonymous inline struct: `nested_params: { .. }`.
    ///
    /// It has no type of its own yet; [`Params::collect`] hoists it into a
    /// named top level struct derived from its parent and field name.
    Inline(Vec<Field>),

    /// An array of structs: `Array<N, OtherStruct>` or `Array<N> { .. }`.
    Array(ArrayField),
}

/// An `Array<N>` / `Array<N, Type>` field.
#[derive(Debug)]
pub struct ArrayField {
    /// The number of elements `N`, as a const expression.
    pub len: Expr,
    /// What a single element is.
    pub element: ArrayElement,
}

/// The element of an [`ArrayField`].
#[derive(Debug)]
pub enum ArrayElement {
    /// `Array<N, Type>`: elements are an explicitly named struct.
    Named(Type),

    /// `Array<N> { .. }`: elements are an anonymous inline struct.
    ///
    /// Like [`FieldValue::Inline`], it gets hoisted into a named struct by
    /// [`Params::collect`].
    Inline(Vec<Field>),
}

/// A leaf field, e.g. `param gain: FloatParam { default: 1f32 }`.
#[derive(Debug)]
pub struct Leaf {
    pub kind: LeafKind,
    pub ty: Type,
    pub entries: Vec<ParamEntry>,
}

/// The kind keyword that introduced a leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafKind {
    /// `param <name>: <Type> { .. }`
    Param,
}

/// A `key: value` entry inside a leaf parameter configuration.
#[derive(Debug)]
pub struct ParamEntry {
    pub key: Ident,
    pub value: Expr,
}

impl Params {
    /// Flatten every struct — including the root `params { .. }` block — into a
    /// list of named structs, hoisting every anonymous group.
    pub(crate) fn collect(self) -> Vec<Struct> {
        let Params { structs, root } = self;

        let mut collected = Vec::new();
        for s in structs {
            collect_struct(s, &mut collected);
        }

        if let Some(root) = root {
            collect_struct(
                Struct {
                    name: parameters_ident(),
                    fields: root.fields,
                },
                &mut collected,
            );
        }

        collected
    }
}

/// Name of the struct produced by the `params { .. }` root block.
pub(crate) fn parameters_ident() -> Ident {
    Ident::new("Parameters", Span::call_site())
}

/// Collect `s`, then all the structs hoisted out of its inline fields.
fn collect_struct(s: Struct, collected: &mut Vec<Struct>) {
    let Struct { name, fields } = s;
    let mut fields_out = Vec::with_capacity(fields.len());
    let mut hoisted = Vec::new();

    for field in fields {
        let Field {
            name: field_name,
            value,
        } = field;
        match value {
            FieldValue::Inline(inner_fields) => {
                let ty = hoist_inline(&name, &field_name, inner_fields, &mut hoisted);
                fields_out.push(Field {
                    name: field_name,
                    value: FieldValue::Reference(ty),
                });
            }
            FieldValue::Array(ArrayField {
                len,
                element: ArrayElement::Inline(inner_fields),
            }) => {
                let ty = hoist_inline(&name, &field_name, inner_fields, &mut hoisted);
                fields_out.push(Field {
                    name: field_name,
                    value: FieldValue::Array(ArrayField {
                        len,
                        element: ArrayElement::Named(ty),
                    }),
                });
            }
            other => fields_out.push(Field {
                name: field_name,
                value: other,
            }),
        }
    }

    collected.push(Struct {
        name,
        fields: fields_out,
    });
    collected.extend(hoisted);
}

/// Hoist an anonymous inline struct into a generated top level struct and
/// return the type that now refers to it.
fn hoist_inline(
    parent: &Ident,
    field_name: &Ident,
    inner_fields: Vec<Field>,
    hoisted: &mut Vec<Struct>,
) -> Type {
    let group_name = inline_struct_name(parent, field_name);
    let group = Struct {
        name: group_name.clone(),
        fields: inner_fields,
    };

    // Recursively hoist, so deeper inline structs get named from `group_name`.
    let mut group_out = Vec::new();
    collect_struct(group, &mut group_out);
    hoisted.extend(group_out);

    syn::parse_quote!(#group_name)
}

/// Build the name of a hoisted inline struct: `OscillatorParam` + `nested_params`
/// becomes `OscillatorParamNestedParams`.
fn inline_struct_name(parent: &Ident, field: &Ident) -> Ident {
    let name = format!("{}{}", parent, to_pascal_case(&field.to_string()));
    Ident::new(&name, field.span())
}

pub(crate) fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

impl Parse for Params {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut structs = Vec::new();
        let mut root = None;

        while !input.is_empty() {
            if input.peek(Token![struct]) {
                structs.push(input.parse()?);
            } else if peek_keyword(input, "params") {
                if root.is_some() {
                    return Err(input.error("only one `params { .. }` block is allowed"));
                }
                root = Some(input.parse()?);
            } else {
                return Err(input.error(
                    "expected a `struct` definition or a `params { .. }` block, e.g. \
                     `struct Oscillator { param gain: FloatParam { default: 0.5 } }`",
                ));
            }
        }

        ensure_unique_struct_names(&structs)?;
        Ok(Self { structs, root })
    }
}

impl Parse for Root {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword = parse_ident(input, "`params`")?;
        let content = braced_content(input, "to open the `params` body")?;
        let fields = parse_fields(&content)?;

        ensure_unique_field_names(&keyword, &fields)?;
        Ok(Self { fields })
    }
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![struct]>()?;

        let name = parse_ident(input, "a struct name (e.g. `MyParams`) after `struct`")?;

        let content = braced_content(input, "to open the struct body")?;
        let fields = parse_fields(&content)?;

        ensure_unique_field_names(&name, &fields)?;
        Ok(Self { name, fields })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let first = parse_ident(input, "a field name")?;

        // `param <name>: <Type> { .. }` (and future leaf kinds).
        if first == "param" {
            let name = parse_ident(input, "a field name after `param`")?;
            expect_colon(input, &name)?;

            let leaf = parse_leaf(input, LeafKind::Param)?;
            return Ok(Self {
                name,
                value: FieldValue::Leaf(leaf),
            });
        }

        expect_colon(input, &first)?;
        let value = parse_structural_value(input)?;
        ensure_non_empty_group(&first, &value)?;
        Ok(Self { name: first, value })
    }
}

/// Parse `param <name>: <Type> { key: value, .. }`, where the config block is
/// optional (`param name: FloatParam` is valid and uses defaults).
fn parse_leaf(input: ParseStream, kind: LeafKind) -> Result<Leaf> {
    if input.is_empty() {
        return Err(input.error("expected a parameter type (e.g. `FloatParam`)"));
    }

    let ty: Type = input
        .parse()
        .map_err(|_| input.error("expected a parameter type (e.g. `FloatParam`)"))?;

    let entries = if input.peek(Brace) {
        let content = braced_content(input, "to open the parameter configuration")?;
        parse_param_entries(&content)?
    } else {
        Vec::new()
    };

    Ok(Leaf { kind, ty, entries })
}

/// Parse a structural field value: an anonymous group, a struct reference, or
/// an array.
fn parse_structural_value(input: ParseStream) -> Result<FieldValue> {
    // Anonymous groups have no type before the braces.
    if input.peek(Brace) {
        let content = braced_content(input, "to open the anonymous group")?;
        return Ok(FieldValue::Inline(parse_fields(&content)?));
    }

    if input.is_empty() {
        return Err(input.error(
            "expected a field value: an anonymous group (`{ .. }`), a struct \
             reference (`OtherStruct`), or `Array<N[, Type]>`",
        ));
    }

    let ty: Type = input.parse().map_err(|_| {
        input.error(
            "expected a field value: an anonymous group (`{ .. }`), a struct \
             reference (`OtherStruct`), or `Array<N[, Type]>`",
        )
    })?;

    // `Array<N, Type>` and `Array<N> { .. }` handling.
    if let Some((len, element)) = split_array_type(&ty) {
        let element = match element {
            // `Array<N, Type>`: inlined name.
            Some(element) => ArrayElement::Named(element),
            // `Array<N> { .. }`: the element type is anonymous, so a body
            // listing the fields is required.
            None => {
                let content = braced_content(input, "to open the anonymous `Array` element body")?;
                ArrayElement::Inline(parse_fields(&content)?)
            }
        };
        return Ok(FieldValue::Array(ArrayField { len, element }));
    }

    // A bare `Type { .. }` used to be a leaf; it now needs a kind keyword.
    if input.peek(Brace) {
        return Err(input.error(
            "expected a field kind before a parameter definition, e.g. \
             `param gain: FloatParam { .. }`",
        ));
    }

    Ok(FieldValue::Reference(ty))
}

fn expect_colon(input: ParseStream, name: &Ident) -> Result<()> {
    if !input.peek(Token![:]) {
        return Err(input.error(format!("expected `:` after field `{name}`")));
    }
    input.parse::<Token![:]>()?;
    Ok(())
}

/// An anonymous group with no fields expands to nothing, which is almost always
/// a typo. Reject it with a clear error instead of silently dropping it.
fn ensure_non_empty_group(name: &Ident, value: &FieldValue) -> Result<()> {
    let fields = match value {
        FieldValue::Inline(fields) => fields,
        FieldValue::Array(ArrayField {
            element: ArrayElement::Inline(fields),
            ..
        }) => fields,
        _ => return Ok(()),
    };

    if fields.is_empty() {
        return Err(syn::Error::new(
            name.span(),
            format!("`{name}` is an empty param group; add at least one field"),
        ));
    }
    Ok(())
}

/// Recognise the special `Array<N>` / `Array<N, Element>` type.
///
/// Returns the length expression and, for the two argument form, the element
/// type. Returns `None` for any other type.
fn split_array_type(ty: &Type) -> Option<(Expr, Option<Type>)> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    if type_path.qself.is_some() {
        return None;
    }

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Array" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let mut args = args.args.iter();
    let len = const_arg_to_expr(args.next()?)?;
    let element = match args.next() {
        None => None,
        Some(GenericArgument::Type(ty)) => Some(ty.clone()),
        Some(_) => return None,
    };

    // Only `Array<N>` and `Array<N, Element>` are valid.
    if args.next().is_some() {
        return None;
    }

    Some((len, element))
}

/// Read a const generic argument, e.g. the `N` in `Array<N, Type>`.
///
/// Literals come through as `GenericArgument::Const`, but a bare const
/// identifier such as `Array<MAX_VOICES>` is parsed by syn as a type, so a
/// plain path is converted back into an expression.
fn const_arg_to_expr(arg: &GenericArgument) -> Option<Expr> {
    match arg {
        GenericArgument::Const(expr) => Some(expr.clone()),
        GenericArgument::Type(Type::Path(type_path)) if type_path.qself.is_none() => {
            Some(Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: type_path.path.clone(),
            }))
        }
        _ => None,
    }
}

fn parse_fields(input: ParseStream) -> Result<Vec<Field>> {
    let mut fields = Vec::new();
    while !input.is_empty() {
        fields.push(input.parse()?);
        // Trailing comma is optional, but fields must be comma separated.
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        } else if !input.is_empty() {
            return Err(input.error("expected `,` between fields"));
        }
    }
    Ok(fields)
}

fn parse_param_entries(input: ParseStream) -> Result<Vec<ParamEntry>> {
    let mut entries = Vec::new();
    while !input.is_empty() {
        let key = parse_ident(input, "a config key (e.g. `default`, `range`)")?;

        if !input.peek(Token![:]) {
            return Err(input.error(format!("expected `:` after config key `{key}`")));
        }
        input.parse::<Token![:]>()?;

        let value: Expr = input
            .parse()
            .map_err(|_| input.error(format!("expected an expression after `{key}:`")))?;
        entries.push(ParamEntry { key, value });

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        } else if !input.is_empty() {
            return Err(input.error("expected `,` between parameter entries"));
        }
    }
    Ok(entries)
}

/// Parse an identifier, reporting `expected` when the next token is not one.
fn parse_ident(input: ParseStream, expected: &str) -> Result<Ident> {
    if input.peek(Ident) {
        input.parse()
    } else {
        Err(input.error(format!("expected {expected}")))
    }
}

/// Check whether the next token is the given (non-keyword) identifier.
fn peek_keyword(input: ParseStream, keyword: &str) -> bool {
    input.peek(Ident) && input.fork().parse::<Ident>().is_ok_and(|id| id == keyword)
}

/// Open a braced group, reporting `context` when the brace is missing.
fn braced_content<'a>(input: ParseStream<'a>, context: &str) -> Result<ParseBuffer<'a>> {
    if !input.peek(Brace) {
        return Err(input.error(format!("expected `{{` {context}")));
    }
    let content;
    braced!(content in input);
    Ok(content)
}

fn ensure_unique_struct_names(structs: &[Struct]) -> Result<()> {
    let mut seen = HashSet::new();
    for s in structs {
        let name = s.name.to_string();
        if !seen.insert(name.clone()) {
            return Err(syn::Error::new(
                s.name.span(),
                format!("struct `{name}` is defined more than once"),
            ));
        }
    }
    Ok(())
}

fn ensure_unique_field_names(struct_name: &Ident, fields: &[Field]) -> Result<()> {
    let mut seen = HashSet::new();
    for field in fields {
        let name = field.name.to_string();
        if !seen.insert(name.clone()) {
            return Err(syn::Error::new(
                field.name.span(),
                format!("field `{name}` is defined more than once in struct `{struct_name}`"),
            ));
        }
    }
    Ok(())
}
