//! Parsing of the `params!` token stream and its lowering into the resolved IR
//! that [`crate::params::expand`] renders.
//!
//! Lowering happens in a single pass: [`Params::resolve`] walks the parse tree,
//! names every anonymous group, and emits the type definitions, the construction
//! tree and the parameter ids together.

use proc_macro2::Span;
use std::collections::{HashMap, HashSet};
use syn::{
    Expr, ExprPath, GenericArgument, Ident, PathArguments, Result, Token, Type, braced,
    parse::{Parse, ParseBuffer, ParseStream},
    token::Brace,
};

// ---------------------------------------------------------------------------
// Syntax tree
// ---------------------------------------------------------------------------

/// Parsed content of `params! { ... }`.
///
/// The grammar is a list of struct definitions followed by an optional `params`
/// root:
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
    /// The fields of the optional `params { .. }` block, which becomes the root
    /// `Parameters` group.
    pub root: Option<Vec<Field>>,
}

/// A named struct definition, e.g. `struct OscillatorParam { .. }`.
#[derive(Debug)]
pub struct Struct {
    pub name: Ident,
    pub fields: Vec<Field>,
}

/// A single field of a struct.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub value: FieldValue,
}

/// The value of a field, which tells us whether we hit a leaf or a group.
#[derive(Debug, Clone)]
pub enum FieldValue {
    /// A leaf introduced by a kind keyword, e.g. `param gain: FloatParam { .. }`.
    ///
    /// Other kinds (`meter`, ...) will be added here later.
    Leaf(Leaf),

    /// A reference to another named struct: `left: ChannelParam`.
    Reference(Type),

    /// An anonymous inline group: `nested: { .. }`, named during lowering from
    /// its parent struct and field name.
    Inline(Vec<Field>),

    /// An array of structs: `Array<N, OtherStruct>` or `Array<N> { .. }`.
    Array(ArrayField),
}

/// An `Array<N>` / `Array<N, Type>` field.
#[derive(Debug, Clone)]
pub struct ArrayField {
    /// The number of elements `N`, as a const expression.
    pub len: Expr,
    /// What a single element is.
    pub element: ArrayElement,
}

/// The element of an [`ArrayField`].
#[derive(Debug, Clone)]
pub enum ArrayElement {
    /// `Array<N, Type>`: elements are an explicitly named struct.
    Named(Type),

    /// `Array<N> { .. }`: elements are an anonymous group, named during lowering.
    Inline(Vec<Field>),
}

/// A leaf field, e.g. `param gain: FloatParam { default: 1f32 }`.
#[derive(Debug, Clone)]
pub struct Leaf {
    /// The kind keyword that introduced this leaf. Only `param` exists today;
    /// upcoming kinds (`meter`, ...) will read it while lowering.
    #[allow(dead_code)]
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
#[derive(Debug, Clone)]
pub struct ParamEntry {
    pub key: Ident,
    pub value: Expr,
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

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
                root = Some(parse_root(input)?);
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

fn parse_root(input: ParseStream) -> Result<Vec<Field>> {
    let keyword = parse_ident(input, "`params`")?;
    let content = braced_content(input, "to open the `params` body")?;
    let fields = parse_fields(&content)?;

    ensure_unique_field_names(&keyword, &fields)?;
    Ok(fields)
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

// ---------------------------------------------------------------------------
// Names
// ---------------------------------------------------------------------------

/// Name of the struct produced by the `params { .. }` root block.
fn parameters_ident() -> Ident {
    Ident::new("Parameters", Span::call_site())
}

/// Build the name of a group: `OscillatorParam` + `nested_params` becomes
/// `OscillatorParamNestedParams`.
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

/// Join a group path with a field name: `oscillator.nested`.
pub(crate) fn join_path(prefix: &str, field: &str) -> String {
    if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{prefix}.{field}")
    }
}

/// The module path of an array element: `field[index]`, where `index` is a
/// literal for an unrolled array or `{}` for a runtime one.
fn indexed_path(parent: &str, field: &Ident, index: &str) -> String {
    format!("{}[{}]", join_path(parent, &field.to_string()), index)
}

/// Join accumulated group names into a single identifier: `nested_pan`.
fn join_name(prefix: &str, field: &str) -> String {
    if prefix.is_empty() {
        field.to_string()
    } else {
        format!("{prefix}_{field}")
    }
}

// ---------------------------------------------------------------------------
// Resolved IR
// ---------------------------------------------------------------------------

/// A fully resolved block: the type definitions, the construction tree and
/// every compile-time id. Rendering it needs no further lookup or walk.
#[derive(Debug)]
pub(crate) struct Resolved {
    /// The path-agnostic type definitions, in source order.
    pub structs: Vec<StructDef>,
    /// The construction tree for `Parameters::create()` (present when a root
    /// `params { .. }` block exists).
    pub root: Option<Construct>,
    /// The `param_ids` module tree: one entry per reachable parameter.
    pub ids: Vec<IdEntry>,
}

/// `struct Name { field: Type, .. }`.
#[derive(Debug)]
pub(crate) struct StructDef {
    pub name: Ident,
    pub fields: Vec<DefField>,
}

/// A field of a [`StructDef`].
#[derive(Debug)]
pub(crate) struct DefField {
    pub name: Ident,
    pub ty: Type,
}

/// A resolved struct construction, e.g. `ChannelParam { gain: .. }`.
#[derive(Debug, Clone)]
pub(crate) struct Construct {
    pub name: Ident,
    pub fields: Vec<Init>,
}

/// One field of a [`Construct`].
#[derive(Debug, Clone)]
pub(crate) struct Init {
    pub name: Ident,
    pub value: InitValue,
}

/// What a field is initialised with.
#[derive(Debug, Clone)]
pub(crate) enum InitValue {
    Leaf(LeafInit),
    Group(Construct),
    Array(ArrayInit),
}

/// A leaf parameter and its inline configuration.
#[derive(Debug, Clone)]
pub(crate) struct LeafInit {
    pub ty: Type,
    pub entries: Vec<ParamEntry>,
}

/// `Array<N, T>`: the construction of a single element, replicated by
/// `core::array::from_fn`.
#[derive(Debug, Clone)]
pub(crate) struct ArrayInit {
    /// The element construction; the field type already carries `N`.
    pub element: Construct,
    /// The declared length `N`. A literal length lets `param_ids` unroll one
    /// module per element; otherwise the index stays a runtime parameter.
    pub len: Expr,
}

/// A node of the `param_ids` module tree, mirroring the parameter groups.
#[derive(Debug)]
pub(crate) enum IdEntry {
    /// A module grouping entries, e.g. `oscillator` or `i1`.
    Module { name: Ident, entries: Vec<IdEntry> },
    /// A single parameter's id.
    Leaf(IdLeaf),
}

/// One parameter's id inside the `param_ids` tree.
#[derive(Debug)]
pub(crate) struct IdLeaf {
    /// `LEFT_GAIN` for a static path, `nested_gain` for an indexed one.
    pub name: Ident,
    /// The `module` handed to `param_id`. For an indexed leaf it is a `format!`
    /// template holding one `{}` per entry in [`IdLeaf::indices`].
    pub module: String,
    /// The parameter's own name.
    pub param: String,
    /// Runtime index parameters, in template order. Empty means the id is a
    /// `const`; otherwise it is a `fn` taking those indices.
    pub indices: Vec<Ident>,
}

// ---------------------------------------------------------------------------
// Lowering
// ---------------------------------------------------------------------------

impl Params {
    /// Lower the parse tree into the IR consumed by [`crate::params::expand`].
    pub(crate) fn resolve(self) -> Result<Resolved> {
        let Params { structs, root } = self;

        let mut lower = Lowerer::new(structs);
        // Defining every top-level struct gives unreferenced ones a type too,
        // naming their anonymous groups along the way.
        lower.define_all()?;

        let root = match root {
            Some(fields) => Some(lower.construct(parameters_ident(), &fields)?),
            None => None,
        };

        let ids = match &root {
            Some(root) => collect_ids(root),
            None => Vec::new(),
        };

        Ok(Resolved {
            structs: lower.definitions(),
            root,
            ids,
        })
    }
}

/// Builds the resolved IR, constructing each struct once and reusing it for
/// every reference.
struct Lowerer {
    /// The top-level structs, addressed through `index`.
    structs: Vec<Struct>,
    index: HashMap<String, usize>,
    /// Names of every constructed struct, in definition order.
    order: Vec<String>,
    cache: HashMap<String, Construct>,
    /// Structs currently being constructed, to catch recursive groups.
    visiting: Vec<String>,
}

impl Lowerer {
    fn new(structs: Vec<Struct>) -> Self {
        let index = structs
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name.to_string(), i))
            .collect();

        Self {
            structs,
            index,
            order: Vec::new(),
            cache: HashMap::new(),
            visiting: Vec::new(),
        }
    }

    /// Construct every top-level struct, so each one and its anonymous groups
    /// get a definition even when nothing references it.
    fn define_all(&mut self) -> Result<()> {
        let top: Vec<(Ident, Vec<Field>)> = self
            .structs
            .iter()
            .map(|s| (s.name.clone(), s.fields.clone()))
            .collect();

        for (name, fields) in top {
            self.construct(name, &fields)?;
        }
        Ok(())
    }

    /// The type definitions, projected from the constructed structs.
    fn definitions(&self) -> Vec<StructDef> {
        self.order
            .iter()
            .map(|name| struct_def(&self.cache[name]))
            .collect()
    }

    /// Construct the struct `name`, or return the cached one.
    fn construct(&mut self, name: Ident, fields: &[Field]) -> Result<Construct> {
        let key = name.to_string();
        if let Some(construct) = self.cache.get(&key) {
            return Ok(construct.clone());
        }
        if self.visiting.iter().any(|visiting| visiting == &key) {
            return Err(syn::Error::new(
                name.span(),
                format!("`{key}` contains itself; a param group cannot be recursive"),
            ));
        }

        self.visiting.push(key.clone());
        self.order.push(key.clone());

        let fields = fields
            .iter()
            .map(|field| self.init(&name, field))
            .collect::<Result<Vec<_>>>()?;

        self.visiting.pop();

        let construct = Construct { name, fields };
        self.cache.insert(key, construct.clone());
        Ok(construct)
    }

    fn init(&mut self, parent: &Ident, field: &Field) -> Result<Init> {
        let value = match &field.value {
            FieldValue::Leaf(leaf) => InitValue::Leaf(LeafInit {
                ty: leaf.ty.clone(),
                entries: leaf.entries.clone(),
            }),

            FieldValue::Reference(ty) => InitValue::Group(self.reference(ty)?),

            FieldValue::Inline(fields) => {
                InitValue::Group(self.construct(inline_struct_name(parent, &field.name), fields)?)
            }

            FieldValue::Array(ArrayField { len, element }) => {
                let element = match element {
                    ArrayElement::Named(ty) => self.reference(ty)?,
                    ArrayElement::Inline(fields) => {
                        self.construct(inline_struct_name(parent, &field.name), fields)?
                    }
                };
                InitValue::Array(ArrayInit {
                    element,
                    len: len.clone(),
                })
            }
        };

        Ok(Init {
            name: field.name.clone(),
            value,
        })
    }

    /// Resolve `ty` to the construction of the struct it names.
    fn reference(&mut self, ty: &Type) -> Result<Construct> {
        let name = struct_name(ty)?;
        if let Some(construct) = self.cache.get(&name) {
            return Ok(construct.clone());
        }

        let Some(&pos) = self.index.get(&name) else {
            return Err(syn::Error::new_spanned(
                ty,
                format!("unknown struct `{name}`; it must be defined in this `params!` block"),
            ));
        };

        let target = self.structs[pos].name.clone();
        let fields = self.structs[pos].fields.clone();
        self.construct(target, &fields)
    }
}

/// `struct Name { field: Type, .. }`, derived from the construction.
fn struct_def(construct: &Construct) -> StructDef {
    StructDef {
        name: construct.name.clone(),
        fields: construct
            .fields
            .iter()
            .map(|init| DefField {
                name: init.name.clone(),
                ty: init_type(&init.value),
            })
            .collect(),
    }
}

/// The Rust type held by a constructed field.
fn init_type(value: &InitValue) -> Type {
    match value {
        InitValue::Leaf(leaf) => leaf.ty.clone(),
        InitValue::Group(group) => named_type(&group.name),
        InitValue::Array(array) => {
            let element = named_type(&array.element.name);
            let len = &array.len;
            syn::parse_quote!([#element; #len])
        }
    }
}

fn named_type(name: &Ident) -> Type {
    syn::parse_quote!(#name)
}

/// The last path segment of a struct reference, e.g. `ChannelParam`.
fn struct_name(ty: &Type) -> Result<String> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(ty, "expected a struct type"));
    };
    type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .ok_or_else(|| syn::Error::new_spanned(ty, "expected a struct type"))
}

// ---------------------------------------------------------------------------
// Ids
// ---------------------------------------------------------------------------

/// Walk the construction tree to build the `param_ids` module tree.
fn collect_ids(root: &Construct) -> Vec<IdEntry> {
    let mut entries = Vec::new();
    walk_ids(root, &IdPath::root(), &mut entries);
    entries
}

fn walk_ids(construct: &Construct, path: &IdPath, out: &mut Vec<IdEntry>) {
    for init in &construct.fields {
        match &init.value {
            InitValue::Leaf(_) => out.push(IdEntry::Leaf(path.leaf(&init.name))),

            InitValue::Group(group) => walk_ids(group, &path.group(&init.name), out),

            InitValue::Array(array) => {
                let entries = match literal_len(&array.len) {
                    Some(len) => (0..len)
                        .map(|i| IdEntry::Module {
                            name: index_module(i),
                            entries: id_entries(&array.element, &path.element(&init.name, i)),
                        })
                        .collect(),
                    None => id_entries(&array.element, &path.indexed_element(&init.name)),
                };

                out.push(IdEntry::Module {
                    name: init.name.clone(),
                    entries,
                });
            }
        }
    }
}

fn id_entries(construct: &Construct, path: &IdPath) -> Vec<IdEntry> {
    let mut entries = Vec::new();
    walk_ids(construct, path, &mut entries);
    entries
}

/// Where the id walk currently sits in the tree.
///
/// A named group only extends the module path and the accumulated id name. An
/// array starts a new module, because its elements are told apart by an index:
/// unrolled when the length is a literal, otherwise passed at runtime.
struct IdPath {
    /// The module string, with one `{}` per runtime index.
    module: String,
    /// Group names gathered since the last array, joined with `_`.
    prefix: String,
    /// Runtime index parameters in scope.
    indices: Vec<Ident>,
}

impl IdPath {
    fn root() -> Self {
        Self {
            module: String::new(),
            prefix: String::new(),
            indices: Vec::new(),
        }
    }

    /// Descend into a named group.
    fn group(&self, field: &Ident) -> Self {
        Self {
            module: join_path(&self.module, &field.to_string()),
            prefix: join_name(&self.prefix, &field.to_string()),
            indices: self.indices.clone(),
        }
    }

    /// Enter element `index` of an array whose length is a literal.
    fn element(&self, field: &Ident, index: usize) -> Self {
        Self {
            module: indexed_path(&self.module, field, &index.to_string()),
            prefix: String::new(),
            indices: self.indices.clone(),
        }
    }

    /// Enter an element of an array whose length is not a literal, turning the
    /// index into a runtime parameter of every id below it.
    fn indexed_element(&self, field: &Ident) -> Self {
        let mut indices = self.indices.clone();
        indices.push(index_var(self.indices.len()));

        Self {
            module: indexed_path(&self.module, field, "{}"),
            prefix: String::new(),
            indices,
        }
    }

    /// The id of the leaf `field`.
    fn leaf(&self, field: &Ident) -> IdLeaf {
        let name = join_name(&self.prefix, &field.to_string());

        IdLeaf {
            name: if self.indices.is_empty() {
                Ident::new(&name.to_ascii_uppercase(), field.span())
            } else {
                Ident::new(&name, field.span())
            },
            module: self.module.clone(),
            param: field.to_string(),
            indices: self.indices.clone(),
        }
    }
}

/// The value of an array length written as an integer literal.
fn literal_len(len: &Expr) -> Option<usize> {
    let Expr::Lit(expr) = len else { return None };
    let syn::Lit::Int(int) = &expr.lit else {
        return None;
    };
    int.base10_parse::<usize>().ok()
}

/// Module name of the element at `index` (`i0`, `i1`, ...).
fn index_module(index: usize) -> Ident {
    Ident::new(&format!("i{index}"), Span::call_site())
}

/// Parameter name of the `nth` runtime index (`index0`, `index1`, ...).
fn index_var(nth: usize) -> Ident {
    Ident::new(&format!("index{nth}"), Span::call_site())
}
