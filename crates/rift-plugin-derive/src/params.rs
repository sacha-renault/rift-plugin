use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
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
///         field: ParamType { key: value, .. },
///         group: { field: ParamType { .. } },
///         other: OtherStructName,
///         many: Array<8, OtherStructName>,
///         anon: Array<8> { field: ParamType { .. } },
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Params {
    /// All the top level `struct` definitions, in source order.
    pub structs: Vec<Struct>,
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
    /// A leaf parameter: a type followed by its inline configuration.
    ///
    /// `frequency: FloatParam { default: 20f32, range: 10f32..20000f32 }`
    Param(Param),

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

/// A leaf parameter definition: its type plus the raw `key: value` pairs.
#[derive(Debug)]
pub struct Param {
    pub ty: Type,
    pub entries: Vec<ParamEntry>,
}

/// A `key: value` entry inside a leaf parameter configuration.
#[derive(Debug)]
pub struct ParamEntry {
    pub key: Ident,
    pub value: Expr,
}

impl Params {
    /// Flatten every struct, hoisting each anonymous inline struct into its own
    /// named top level struct.
    ///
    /// A field named `nested_params` inside `OscillatorParam` becomes a
    /// `nested_params: OscillatorParamNestedParams` reference, and a new
    /// `struct OscillatorParamNestedParams { .. }` is emitted right after its
    /// parent. Nested inline structs are handled recursively, so their name is
    /// built from the (possibly generated) parent name.
    pub fn collect(self) -> Vec<Struct> {
        let mut collected = Vec::new();
        for s in self.structs {
            collect_struct(s, &mut collected);
        }
        collected
    }
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

fn to_pascal_case(s: &str) -> String {
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

impl Params {
    /// Expand every collected struct into its definition plus its constructors.
    ///
    /// Ids are handed out through a shared `cursor` so that nested groups and
    /// every element of an `Array<N, T>` get distinct ids, without the macro
    /// having to know how many params a struct holds.
    pub fn expand(self) -> TokenStream2 {
        let structs = self.collect().into_iter().map(expand_struct);
        quote::quote! { #( #structs )* }
    }
}

fn expand_struct(s: Struct) -> TokenStream2 {
    let name = &s.name;
    let field_names: Vec<_> = s.fields.iter().map(|f| &f.name).collect();
    let field_types: Vec<_> = s.fields.iter().map(field_type).collect();
    let field_inits: Vec<_> = s.fields.iter().map(field_init).collect();

    quote::quote! {
        #[allow(dead_code)]
        struct #name {
            #( #field_names: #field_types, )*
        }

        #[allow(dead_code, unused_variables)]
        impl #name {
            fn create() -> Self {
                let mut cursor = 0u32;
                Self::create_from(&mut cursor)
            }

            fn create_from(cursor: &mut u32) -> Self {
                Self {
                    #( #field_names: #field_inits, )*
                }
            }
        }
    }
}

/// The Rust type of a generated struct field.
fn field_type(field: &Field) -> TokenStream2 {
    match &field.value {
        FieldValue::Param(Param { ty, .. }) => quote::quote! { #ty },
        FieldValue::Reference(ty) => quote::quote! { #ty },
        FieldValue::Array(ArrayField {
            len,
            element: ArrayElement::Named(ty),
        }) => quote::quote! { [#ty; #len] },
        FieldValue::Array(ArrayField {
            element: ArrayElement::Inline(_),
            ..
        }) => unreachable!("anonymous `Array` element is hoisted during collect"),
        FieldValue::Inline(_) => unreachable!("anonymous group is hoisted during collect"),
    }
}

/// The expression used to build a generated struct field in `create_from`.
fn field_init(field: &Field) -> TokenStream2 {
    match &field.value {
        FieldValue::Param(Param { ty, entries }) => {
            let param = type_path_expr(ty);
            let config = type_path_expr(&config_type(ty));
            let name = syn::LitStr::new(&field.name.to_string(), field.name.span());
            let keys: Vec<_> = entries.iter().map(|e| &e.key).collect();
            let values: Vec<_> = entries.iter().map(|e| &e.value).collect();

            quote::quote! {
                {
                    let id = ::rift_plugin::prelude::clack_plugin::prelude::ClapId::new(*cursor);
                    *cursor += 1u32;
                    #param::create(id, #name.to_string(), None, #config {
                        #( #keys: #values, )*
                        ..Default::default()
                    })
                }
            }
        }
        FieldValue::Reference(ty) => {
            let param = type_path_expr(ty);
            quote::quote! { #param::create_from(cursor) }
        }
        FieldValue::Array(ArrayField {
            element: ArrayElement::Named(ty),
            ..
        }) => {
            let param = type_path_expr(ty);
            quote::quote! { ::core::array::from_fn(|_| #param::create_from(cursor)) }
        }
        FieldValue::Array(ArrayField {
            element: ArrayElement::Inline(_),
            ..
        }) => unreachable!("anonymous `Array` element is hoisted during collect"),
        FieldValue::Inline(_) => unreachable!("anonymous group is hoisted during collect"),
    }
}

/// Build the `...Config` type for a leaf param type.
///
/// `FloatParam` -> `FloatParamConfig`,
/// `EnumParam<WaveType>` -> `EnumParamConfig<WaveType>`.
fn config_type(ty: &Type) -> Type {
    let Type::Path(type_path) = ty else {
        return ty.clone();
    };

    let mut type_path = type_path.clone();
    if let Some(segment) = type_path.path.segments.last_mut() {
        let ident = &segment.ident;
        segment.ident = Ident::new(&format!("{ident}Config"), ident.span());
    }
    Type::Path(type_path)
}

/// Render a type path as an expression path, adding turbofish `::` on generic
/// arguments so it can be used in value position (`Foo::<Bar>::create()`).
fn type_path_expr(ty: &Type) -> TokenStream2 {
    let Type::Path(type_path) = ty else {
        return quote::quote! { #ty };
    };

    let mut path = type_path.path.clone();
    for segment in path.segments.iter_mut() {
        if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = Some(Default::default());
        }
    }
    quote::quote! { #path }
}

pub fn proc_params(input: TokenStream) -> TokenStream {
    let params = match syn::parse::<Params>(input) {
        Ok(params) => params,
        Err(e) => return e.into_compile_error().into(),
    };

    params.expand().into()
}

impl Parse for Params {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut structs = Vec::new();

        while !input.is_empty() {
            if !input.peek(Token![struct]) {
                return Err(input.error(
                    "expected a `struct` definition, e.g. \
                     `struct MyParams { gain: FloatParam { default: 0.5, range: 0.0..1.0 } }`",
                ));
            }
            structs.push(input.parse()?);
        }

        ensure_unique_struct_names(&structs)?;
        Ok(Self { structs })
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
        let name = parse_ident(input, "a field name")?;

        if !input.peek(Token![:]) {
            return Err(input.error(format!("expected `:` after field `{name}`")));
        }
        input.parse::<Token![:]>()?;

        let value = parse_field_value(input)?;
        ensure_non_empty_group(&name, &value)?;
        Ok(Self { name, value })
    }
}

fn parse_field_value(input: ParseStream) -> Result<FieldValue> {
    // Anonymous params has no type before the braces.
    if input.peek(Brace) {
        let content = braced_content(input, "to open the anonymous group")?;
        return Ok(FieldValue::Inline(parse_fields(&content)?));
    }

    if input.is_empty() {
        return Err(input.error(
            "expected a field value: a param type (`FloatParam { .. }`), an anonymous \
             group (`{ .. }`), a struct reference (`OtherStruct`), or `Array<N[, Type]>`",
        ));
    }

    let ty: Type = input.parse().map_err(|_| {
        input.error(
            "expected a field value: a param type (`FloatParam`), an anonymous group \
             (`{ .. }`), a struct reference (`OtherStruct`), or `Array<N[, Type]>`",
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

    // `frequency: FloatParam { .. }` is a leaf, `left: ChannelParam` is a
    // reference to another named struct.
    if input.peek(Brace) {
        let content = braced_content(input, "to open the parameter configuration")?;
        let entries = parse_param_entries(&content)?;
        Ok(FieldValue::Param(Param { ty, entries }))
    } else {
        Ok(FieldValue::Reference(ty))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const DEV_INPUT: &str = r#"
        struct OscillatorParam {
            frequency: FloatParam {
                default: 20f32,
                range: 10f32..20000f32,
            },
            wave: EnumParam<WaveType> {
                default: WaveType::Square,
            },
            nested_params: {
                gain: FloatParam {
                    default: 1f32,
                    range: 0f32..2f32,
                },
                pan: FloatParam {
                    default: 0f32,
                    range: -1f32..1f32,
                },
            }
        }

        struct ChannelParam {
            gain: FloatParam {
                default: 1f32,
                range: 0f32..2f32,
            },
        }
    "#;

    fn parse(input: &str) -> Params {
        syn::parse_str::<Params>(input).expect("should parse")
    }

    /// Render any token streamable node for assertions.
    fn ts<T: quote::ToTokens>(node: &T) -> String {
        quote::quote!(#node).to_string()
    }

    #[test]
    fn parses_top_level_structs() {
        let params = parse(DEV_INPUT);
        let names: Vec<_> = params.structs.iter().map(|s| s.name.to_string()).collect();
        assert_eq!(names, ["OscillatorParam", "ChannelParam"]);
    }

    #[test]
    fn parses_leaf_params_with_entries() {
        let params = parse(DEV_INPUT);
        let osc = &params.structs[0];

        let FieldValue::Param(frequency) = &osc.fields[0].value else {
            panic!("frequency should be a leaf param");
        };
        assert_eq!(ts(&frequency.ty), "FloatParam");
        let keys: Vec<_> = frequency
            .entries
            .iter()
            .map(|e| e.key.to_string())
            .collect();
        assert_eq!(keys, ["default", "range"]);

        let FieldValue::Param(wave) = &osc.fields[1].value else {
            panic!("wave should be a leaf param");
        };
        assert_eq!(ts(&wave.ty), "EnumParam < WaveType >");

        let FieldValue::Inline(nested) = &osc.fields[2].value else {
            panic!("nested_params should be inline");
        };
        assert_eq!(nested.len(), 2);
    }

    #[test]
    fn collect_hoists_inline_structs() {
        let collected = parse(DEV_INPUT).collect();
        let names: Vec<_> = collected.iter().map(|s| s.name.to_string()).collect();
        assert_eq!(
            names,
            [
                "OscillatorParam",
                "OscillatorParamNestedParams",
                "ChannelParam",
            ]
        );

        // The inline field now points at the hoisted struct.
        let osc = &collected[0];
        let FieldValue::Reference(ty) = &osc.fields[2].value else {
            panic!("nested_params should now be a reference");
        };
        assert_eq!(ts(ty), "OscillatorParamNestedParams");

        // And the hoisted struct kept the original fields.
        let nested = &collected[1];
        let fields: Vec<_> = nested.fields.iter().map(|f| f.name.to_string()).collect();
        assert_eq!(fields, ["gain", "pan"]);
    }

    #[test]
    fn parses_reference_field() {
        let params = parse(
            r#"
            struct Group {
                left: ChannelParam,
            }
        "#,
        );
        let FieldValue::Reference(ty) = &params.structs[0].fields[0].value else {
            panic!("left should be a reference");
        };
        assert_eq!(ts(ty), "ChannelParam");
    }

    #[test]
    fn parses_named_array() {
        let params = parse(
            r#"
            struct Group {
                voices: Array<4, ChannelParam>,
            }
        "#,
        );
        let FieldValue::Array(array) = &params.structs[0].fields[0].value else {
            panic!("voices should be an array");
        };
        assert_eq!(ts(&array.len), "4");

        let ArrayElement::Named(ty) = &array.element else {
            panic!("element should be a named type");
        };
        assert_eq!(ts(ty), "ChannelParam");
    }

    #[test]
    fn parses_const_ident_array_length() {
        let params = parse(
            r#"
            struct Group {
                voices: Array<MAX_VOICES, ChannelParam>,
            }
        "#,
        );
        let FieldValue::Array(array) = &params.structs[0].fields[0].value else {
            panic!("voices should be an array");
        };
        assert_eq!(ts(&array.len), "MAX_VOICES");
    }

    #[test]
    fn collects_anonymous_array_elements() {
        let collected = parse(
            r#"
            struct EnvelopeParam {
                attack: FloatParam { default: 0.1f32 },
                stages: Array<3> {
                    level: FloatParam { default: 1f32 },
                },
            }
        "#,
        )
        .collect();

        let names: Vec<_> = collected.iter().map(|s| s.name.to_string()).collect();
        assert_eq!(names, ["EnvelopeParam", "EnvelopeParamStages"]);

        // The anonymous element type was hoisted and the array still tracks its
        // length and now points at the generated struct.
        let FieldValue::Array(array) = &collected[0].fields[1].value else {
            panic!("stages should stay an array");
        };
        assert_eq!(ts(&array.len), "3");
        let ArrayElement::Named(ty) = &array.element else {
            panic!("element should now be a named struct");
        };
        assert_eq!(ts(ty), "EnvelopeParamStages");

        let fields: Vec<_> = collected[1]
            .fields
            .iter()
            .map(|f| f.name.to_string())
            .collect();
        assert_eq!(fields, ["level"]);
    }

    #[test]
    fn pascal_case_helper() {
        assert_eq!(to_pascal_case("nested_params"), "NestedParams");
        assert_eq!(to_pascal_case("gain"), "Gain");
        assert_eq!(to_pascal_case("a_b_c"), "ABC");
    }

    fn parse_err(input: &str) -> String {
        syn::parse_str::<Params>(input)
            .expect_err("should fail to parse")
            .to_string()
    }

    #[test]
    fn reports_missing_struct_keyword() {
        let msg = parse_err("Foo { a: FloatParam }");
        assert!(msg.contains("expected a `struct` definition"), "{msg}");
    }

    #[test]
    fn reports_missing_struct_brace() {
        let msg = parse_err("struct Foo;");
        assert!(
            msg.contains("expected `{` to open the struct body"),
            "{msg}"
        );
    }

    #[test]
    fn reports_missing_field_value() {
        let msg = parse_err("struct Foo { gain: }");
        assert!(msg.contains("expected a field value"), "{msg}");
    }

    #[test]
    fn reports_missing_colon() {
        let msg = parse_err("struct Foo { gain FloatParam }");
        assert!(msg.contains("expected `:` after field `gain`"), "{msg}");
    }

    #[test]
    fn reports_missing_comma_between_fields() {
        let msg = parse_err("struct Foo { a: FloatParam b: FloatParam }");
        assert!(msg.contains("expected `,` between fields"), "{msg}");
    }

    #[test]
    fn reports_missing_array_body() {
        let msg = parse_err("struct Foo { stages: Array<3> }");
        assert!(
            msg.contains("expected `{` to open the anonymous `Array` element body"),
            "{msg}"
        );
    }

    #[test]
    fn reports_duplicate_struct() {
        let msg = parse_err("struct Foo {} struct Foo {}");
        assert!(
            msg.contains("struct `Foo` is defined more than once"),
            "{msg}"
        );
    }

    #[test]
    fn reports_duplicate_field() {
        let msg = parse_err("struct Foo { a: FloatParam, a: FloatParam }");
        assert!(
            msg.contains("field `a` is defined more than once in struct `Foo`"),
            "{msg}"
        );
    }

    #[test]
    fn reports_empty_anonymous_group() {
        let msg = parse_err("struct Foo { empty_param: {} }");
        assert!(
            msg.contains("`empty_param` is an empty param group"),
            "{msg}"
        );
    }

    #[test]
    fn reports_empty_array_element_group() {
        let msg = parse_err("struct Foo { stages: Array<3> {} }");
        assert!(msg.contains("`stages` is an empty param group"), "{msg}");
    }

    /// Collapse all whitespace so token-stream assertions are stable.
    fn normalized(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn expands_structs_with_create() {
        let params = parse(
            r#"
            struct Child {
                gain: FloatParam { default: 1f32 },
            }

            struct Parent {
                freq: FloatParam { default: 20f32, range: 10f32..20000f32 },
                child: Child,
                voices: Array<2, Child>,
            }
        "#,
        );

        let out = normalized(&params.expand().to_string());

        assert!(
            out.contains("structParent{freq:FloatParam,child:Child,voices:[Child;2],}"),
            "{out}"
        );
        assert!(
            out.contains("fncreate()->Self{letmutcursor=0u32;Self::create_from(&mutcursor)}"),
            "{out}"
        );
        assert!(out.contains("fncreate_from(cursor:&mutu32)->Self"), "{out}");
        assert!(
            out.contains(
                "FloatParam::create(id,\"freq\".to_string(),None,FloatParamConfig{default:20f32,range:10f32..20000f32,..Default::default()})"
            ),
            "{out}"
        );
        assert!(out.contains("ClapId::new(*cursor)"), "{out}");
        assert!(out.contains("*cursor+=1u32"), "{out}");
        assert!(out.contains("Child::create_from(cursor)"), "{out}");
        assert!(
            out.contains("::core::array::from_fn(|_|Child::create_from(cursor))"),
            "{out}"
        );
    }

    #[test]
    fn expands_hoisted_groups() {
        let params = parse(
            r#"
            struct Envelope {
                stages: Array<3> {
                    level: FloatParam { default: 1f32 },
                },
            }
        "#,
        );

        let out = normalized(&params.expand().to_string());

        assert!(
            out.contains("structEnvelopeStages{level:FloatParam,}"),
            "{out}"
        );
        assert!(
            out.contains("structEnvelopeStages{level:FloatParam,}"),
            "{out}"
        );
        assert!(out.contains("stages:[EnvelopeStages;3],"), "{out}");
        assert!(
            out.contains("::core::array::from_fn(|_|EnvelopeStages::create_from(cursor))"),
            "{out}"
        );
    }

    #[test]
    fn expands_generic_param_with_turbofish() {
        let params =
            parse("struct Foo { wave: EnumParam<WaveType> { default: WaveType::Square } }");
        let out = normalized(&params.expand().to_string());

        assert!(
            out.contains(
                "EnumParam::<WaveType>::create(id,\"wave\".to_string(),None,EnumParamConfig::<WaveType>{default:WaveType::Square,..Default::default()})"
            ),
            "{out}"
        );
    }
}
