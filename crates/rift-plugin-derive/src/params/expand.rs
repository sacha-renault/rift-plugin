//! Rendering of the resolved IR into Rust code.
//!
//! This module is a pure code generator: [`expand`] takes the [`Resolved`] tree
//! produced by [`super::collect`] and turns it into type definitions, the
//! `Parameters::new()` tree and the `param_ids` module. It never looks up a
//! struct by name and never computes ids, resolution already did both.

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Ident, PathArguments, Type};

use super::collect::{
    Construct, DefField, IdEntry, IdLeaf, Init, InitValue, LeafInit, Resolved, StructDef, join_path,
};

/// Render a resolved block.
pub(crate) fn expand(resolved: Resolved) -> TokenStream2 {
    let Resolved { structs, root, ids } = resolved;

    let structs = structs.iter().map(expand_struct_def);
    let ids_impl = expand_param_ids(&ids);
    let new_impl = root.as_ref().map(expand_create);
    let default_impl = root.as_ref().map(expand_default);
    let params_impl = if let Some(root) = root.as_ref() {
        Some(expand_params(root))
    } else {
        None
    };

    quote::quote! {
        #( #structs )*
        #ids_impl
        #new_impl
        #default_impl
        #params_impl
    }
}

/// How the current module path is referred to while emitting a struct value.
///
/// The path is a literal when no array index is involved. As soon as an array
/// index enters it, the path can only be built at runtime, so it is either a
/// `String` expression waiting to be bound ([`Module::Expr`]) or a local binding
/// already in scope ([`Module::Local`]).
enum Module {
    Literal(String),
    Expr(TokenStream2),
    Local(Ident),
}

impl Module {
    /// The new module path of a child field, optionally behind an array index.
    fn child(&self, field: &Ident, index: Option<&Ident>) -> Module {
        let field = field.to_string();

        match (self, index) {
            (Module::Literal(path), None) => Module::Literal(join_path(path, &field)),
            (Module::Literal(path), Some(index)) => {
                let prefix = syn::LitStr::new(&join_path(path, &field), Span::call_site());
                Module::Expr(quote::quote! { format!("{}[{}]", #prefix, #index) })
            }
            (_, None) => {
                let parent = self.render();
                let field = syn::LitStr::new(&field, Span::call_site());
                Module::Expr(quote::quote! { format!("{}.{}", #parent, #field) })
            }
            (_, Some(index)) => {
                let parent = self.render();
                let field = syn::LitStr::new(&field, Span::call_site());
                Module::Expr(quote::quote! { format!("{}.{}[{}]", #parent, #field, #index) })
            }
        }
    }

    /// Bind an expression path to a local so it is built once per struct, then
    /// used by every field.
    fn bind(self, depth: usize) -> (TokenStream2, Module) {
        match self {
            Module::Expr(expr) => {
                let var = module_var(depth);
                (
                    quote::quote! { let #var: String = #expr; },
                    Module::Local(var),
                )
            }
            other => (TokenStream2::new(), other),
        }
    }

    /// The path as a `&str`, for hashing into an id.
    fn as_str(&self) -> TokenStream2 {
        match self {
            Module::Literal(path) => {
                let path = syn::LitStr::new(path, Span::call_site());
                quote::quote! { #path }
            }
            Module::Expr(expr) => quote::quote! { (#expr).as_str() },
            Module::Local(var) => quote::quote! { #var.as_str() },
        }
    }

    /// The path as an `Option<String>`, for `Param::new`.
    fn as_option(&self) -> TokenStream2 {
        match self {
            Module::Literal(path) if path.is_empty() => quote::quote! { None },
            Module::Literal(path) => {
                let path = syn::LitStr::new(path, Span::call_site());
                quote::quote! { Some(#path.to_string()) }
            }
            Module::Expr(expr) => quote::quote! { Some(#expr) },
            Module::Local(var) => quote::quote! { Some(#var.clone()) },
        }
    }

    /// The path as an expression usable inside a `format!`.
    fn render(&self) -> TokenStream2 {
        match self {
            Module::Literal(path) => {
                let path = syn::LitStr::new(path, Span::call_site());
                quote::quote! { #path }
            }
            Module::Expr(expr) => expr.clone(),
            Module::Local(var) => quote::quote! { #var },
        }
    }
}

/// `#[allow(dead_code)] struct Name { field: Type, .. }`.
fn expand_struct_def(s: &StructDef) -> TokenStream2 {
    let name = &s.name;
    let field_names: Vec<_> = s.fields.iter().map(|DefField { name, .. }| name).collect();
    let field_types: Vec<_> = s.fields.iter().map(|DefField { ty, .. }| ty).collect();

    quote::quote! {
        #[allow(dead_code)]
        struct #name {
            #( #field_names: #field_types, )*
        }
    }
}

/// `impl Parameters { fn create() -> Self { .. } }`.
fn expand_create(root: &Construct) -> TokenStream2 {
    let name = &root.name;
    let body = expand_construct(root, Module::Literal(String::new()), 0);

    quote::quote! {
        #[allow(dead_code)]
        impl #name {
            fn new() -> Self {
                #body
            }
        }
    }
}

fn expand_default(root: &Construct) -> TokenStream2 {
    let name = &root.name;

    quote::quote! {
        #[allow(dead_code)]
        impl ::core::default::Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}

/// `impl Params for Name { .. }`.
///
/// Every method walks the leaf parameters in declaration order, descending into
/// arrays with a `for` loop so literal and runtime lengths are handled the same
/// way. Lookups compare against each param's own [`Param::id`], which `create()`
/// already populated from the resolved `param_ids`, so the `ids` tree is not
/// needed here.
fn expand_params(root: &Construct) -> TokenStream2 {
    let name = &root.name;

    let params_trait = quote::quote! { ::rift_plugin::prelude::Params };
    let param_trait = quote::quote! { ::rift_plugin::prelude::Param };
    let clap_id = quote::quote! { ::rift_plugin::prelude::clack_plugin::prelude::ClapId };
    let param_info = quote::quote! { ::rift_plugin::prelude::clack_extensions::params::ParamInfo };
    let display_writer =
        quote::quote! { ::rift_plugin::prelude::clack_extensions::params::ParamDisplayWriter };
    let plugin_error = quote::quote! { ::rift_plugin::prelude::PluginError };

    let mut count = |_leaf: &TokenStream2| quote::quote! { __count += 1; };

    let mut info = |leaf: &TokenStream2| {
        quote::quote! {
            if __index == index {
                return Some(#param_trait::param_info(#leaf));
            }
            __index += 1;
        }
    };

    let mut get = |leaf: &TokenStream2| {
        quote::quote! {
            if #param_trait::id(#leaf) == id {
                return Some(#param_trait::get_raw(#leaf));
            }
        }
    };

    let mut set = |leaf: &TokenStream2| {
        quote::quote! {
            if #param_trait::id(#leaf) == id {
                #param_trait::set_raw(#leaf, value);
                return;
            }
        }
    };

    let mut set_normalized = |leaf: &TokenStream2| {
        quote::quote! {
            if #param_trait::id(#leaf) == id {
                #param_trait::set_normalized(#leaf, value);
                return;
            }
        }
    };

    let mut text_to_value = |leaf: &TokenStream2| {
        quote::quote! {
            if #param_trait::id(#leaf) == id {
                return #param_trait::text_to_value(#leaf, text);
            }
        }
    };

    let mut value_to_text = |leaf: &TokenStream2| {
        quote::quote! {
            if #param_trait::id(#leaf) == id {
                return #param_trait::value_to_text(#leaf, value, writer);
            }
        }
    };

    let access = quote::quote! { (*self) };
    let count = expand_leaves(root, &access, 0, &mut count);
    let info = expand_leaves(root, &access, 0, &mut info);
    let get = expand_leaves(root, &access, 0, &mut get);
    let set = expand_leaves(root, &access, 0, &mut set);
    let set_normalized = expand_leaves(root, &access, 0, &mut set_normalized);
    let text_to_value = expand_leaves(root, &access, 0, &mut text_to_value);
    let value_to_text = expand_leaves(root, &access, 0, &mut value_to_text);

    quote::quote! {
        impl #params_trait for #name {
            fn count(&self) -> u32 {
                let mut __count: u32 = 0;
                #count
                __count
            }

            fn get_param_info<'__a>(&'__a self, index: u32) -> Option<#param_info<'__a>> {
                let mut __index: u32 = 0;
                #info
                None
            }

            fn get_value(&self, id: #clap_id) -> Option<f32> {
                #get
                None
            }

            fn set_value(&self, id: #clap_id, value: f32) {
                #set
            }

            fn set_value_normalized(&self, id: #clap_id, value: f32) {
                #set_normalized
            }

            fn text_to_value(&self, id: #clap_id, text: &::core::ffi::CStr) -> Option<f32> {
                #text_to_value
                None
            }

            fn value_to_text(
                &self,
                id: #clap_id,
                value: f32,
                writer: &mut #display_writer,
            ) -> ::core::fmt::Result {
                #value_to_text
                Err(::core::fmt::Error)
            }

            fn serialize(&self, _writer: &mut dyn ::std::io::Write) -> Result<(), #plugin_error> {
                todo!("Params::serialize is not implemented yet")
            }

            fn deserialize(&self, _reader: &mut dyn ::std::io::Read) -> Result<(), #plugin_error> {
                todo!("Params::deserialize is not implemented yet")
            }
        }
    }
}

/// Run `body` once per leaf parameter, passing the leaf's reference expression
/// (already a `&ConcreteParam`, coerced to `&dyn Param` where needed).
///
/// `access` is a parenthesised place expression of the enclosing struct's type.
/// Named groups extend it; arrays are traversed with a `for` loop, so a runtime
/// length is no different from a literal one.
fn expand_leaves(
    construct: &Construct,
    access: &TokenStream2,
    depth: usize,
    body: &mut dyn FnMut(&TokenStream2) -> TokenStream2,
) -> TokenStream2 {
    let mut statements = TokenStream2::new();

    for init in &construct.fields {
        let field = &init.name;

        match &init.value {
            InitValue::Leaf(_) => {
                let leaf = quote::quote! { &#access.#field };
                statements.extend(body(&leaf));
            }

            InitValue::Group(group) => {
                let child = quote::quote! { (#access.#field) };
                statements.extend(expand_leaves(group, &child, depth, body));
            }

            InitValue::Array(array) => {
                let item = Ident::new(&format!("__item{depth}"), Span::call_site());
                let element = quote::quote! { (*#item) };
                let inner = expand_leaves(&array.element, &element, depth + 1, body);

                statements.extend(quote::quote! {
                    for #item in #access.#field.iter() {
                        #inner
                    }
                });
            }
        }
    }

    statements
}

/// `pub mod param_ids { .. }`, mirroring the parameter groups.
fn expand_param_ids(entries: &[IdEntry]) -> TokenStream2 {
    if entries.is_empty() {
        return TokenStream2::new();
    }

    let items = entries.iter().map(expand_id_entry);

    quote::quote! {
        #[allow(dead_code)]
        pub mod param_ids {
            #( #items )*
        }
    }
}

fn expand_id_entry(entry: &IdEntry) -> TokenStream2 {
    match entry {
        IdEntry::Module { name, entries } => {
            let items = entries.iter().map(expand_id_entry);
            quote::quote! {
                pub mod #name {
                    #( #items )*
                }
            }
        }
        IdEntry::Leaf(leaf) => expand_id_leaf(leaf),
    }
}

/// `pub const LEFT_GAIN: ClapId = param_id("left", "gain");`, or a `fn` taking
/// the runtime indices when the path crosses an array of unknown length.
fn expand_id_leaf(leaf: &IdLeaf) -> TokenStream2 {
    let name = &leaf.name;
    let module = syn::LitStr::new(&leaf.module, leaf.name.span());
    let param = syn::LitStr::new(&leaf.param, leaf.name.span());
    let clap_id = quote::quote! { ::rift_plugin::prelude::clack_plugin::prelude::ClapId };

    if leaf.indices.is_empty() {
        quote::quote! {
            pub const #name: #clap_id = ::rift_plugin::prelude::param_id(#module, #param);
        }
    } else {
        let indices = &leaf.indices;
        quote::quote! {
            pub fn #name(#( #indices: usize, )*) -> #clap_id {
                ::rift_plugin::prelude::param_id(&format!(#module, #( #indices ),*), #param)
            }
        }
    }
}

/// `Name { field: value, .. }`, with the module path in scope for its fields.
fn expand_construct(construct: &Construct, module: Module, depth: usize) -> TokenStream2 {
    let (binding, module) = module.bind(depth);

    let field_names: Vec<_> = construct
        .fields
        .iter()
        .map(|Init { name, .. }| name)
        .collect();
    let field_values: Vec<_> = construct
        .fields
        .iter()
        .map(|init| expand_init(init, &module, depth))
        .collect();

    let name = &construct.name;
    let value = quote::quote! {
        #name {
            #( #field_names: #field_values, )*
        }
    };

    if binding.is_empty() {
        value
    } else {
        quote::quote! { { #binding #value } }
    }
}

/// The value of a single field.
fn expand_init(init: &Init, module: &Module, depth: usize) -> TokenStream2 {
    match &init.value {
        // A leaf belongs to the module of the group that holds it.
        InitValue::Leaf(leaf) => expand_leaf(&init.name, leaf, module),

        InitValue::Group(group) => {
            let child = module.child(&init.name, None);
            expand_construct(group, child, depth)
        }

        InitValue::Array(array) => {
            let index = array_index(depth);
            let child = module.child(&init.name, Some(&index));
            let element = expand_construct(&array.element, child, depth + 1);
            quote::quote! { ::core::array::from_fn(|#index| #element) }
        }
    }
}

/// `ParamType::create(param_id(module, name), "name", module, Config { .. })`.
fn expand_leaf(name: &Ident, leaf: &LeafInit, module: &Module) -> TokenStream2 {
    let param = type_path_expr(&leaf.ty);
    let config = type_path_expr(&config_type(&leaf.ty));
    let name_str = syn::LitStr::new(&name.to_string(), name.span());
    let module_str = module.as_str();
    let module_arg = module.as_option();
    let keys: Vec<_> = leaf.entries.iter().map(|entry| &entry.key).collect();
    let values: Vec<_> = leaf.entries.iter().map(|entry| &entry.value).collect();

    quote::quote! {
        #param::create(
            ::rift_plugin::prelude::param_id(#module_str, #name_str),
            #name_str.to_string(),
            #module_arg,
            #config {
                #( #keys: #values, )*
                ..Default::default()
            },
        )
    }
}

/// Local variable holding a runtime module path at `depth`.
///
/// Deeper structs shadow shallower ones, which is fine: a binding's initialiser
/// still sees the outer variable of the same name.
fn module_var(depth: usize) -> Ident {
    Ident::new(&format!("__module{depth}"), Span::call_site())
}

/// Index variable for the array at `depth` (`i0`, `i1`, ...).
fn array_index(depth: usize) -> Ident {
    Ident::new(&format!("i{depth}"), Span::call_site())
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
