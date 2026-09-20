//! Expansion of the collected structs into Rust code.
//!
//! Every struct becomes a path-agnostic type definition. The root
//! `params { .. }` block additionally gets a `create()` that inlines the whole
//! tree, assigning ids through a shared cursor and building full module paths.

use proc_macro2::{Span, TokenStream as TokenStream2};
use std::collections::HashMap;
use syn::{Ident, PathArguments, Result, Type};

use super::collect::{
    ArrayElement, ArrayField, Field, FieldValue, Leaf, LeafKind, Params, Struct, parameters_ident,
};

impl Params {
    /// Expand every struct into its type definition, plus a `create()` on the
    /// root `Parameters` struct (when a `params { .. }` block is present).
    ///
    /// Only the root knows the full tree, so it owns the ids and module paths.
    pub(crate) fn expand(self) -> Result<TokenStream2> {
        let has_root = self.root.is_some();
        let collected = self.collect();

        let registry: HashMap<String, &Struct> =
            collected.iter().map(|s| (s.name.to_string(), s)).collect();

        let definitions = collected.iter().map(expand_struct_def);
        let create = if has_root {
            let root = registry
                .get(&parameters_ident().to_string())
                .expect("the `params { .. }` block produces a `Parameters` struct");
            Some(expand_create(root, &registry)?)
        } else {
            None
        };

        Ok(quote::quote! {
            #( #definitions )*
            #create
        })
    }
}

/// `struct Name { .. }` — a path-agnostic type definition.
fn expand_struct_def(s: &Struct) -> TokenStream2 {
    let name = &s.name;
    let field_names: Vec<_> = s.fields.iter().map(|f| &f.name).collect();
    let field_types: Vec<_> = s.fields.iter().map(field_type).collect();

    quote::quote! {
        #[allow(dead_code)]
        struct #name {
            #( #field_names: #field_types, )*
        }
    }
}

/// `impl Parameters { fn create() -> Self { .. } }`, inlining the whole tree.
fn expand_create(root: &Struct, registry: &HashMap<String, &Struct>) -> Result<TokenStream2> {
    let name = &root.name;
    let body = expand_struct_value(root, &None, registry, 0)?;

    Ok(quote::quote! {
        #[allow(dead_code)]
        impl #name {
            fn create() -> Self {
                let mut cursor = 0u32;
                #body
            }
        }
    })
}

/// Inline-construct a struct value, threading `cursor` for ids and building the
/// `module` path from the root.
///
/// `module` is `None` at the root, otherwise a `String` expression holding the
/// group path (e.g. `oscillator[0].nested_params`). `depth` gives each array
/// closure a unique index variable (`i0`, `i1`, ...).
fn expand_struct_value(
    s: &Struct,
    module: &Option<TokenStream2>,
    registry: &HashMap<String, &Struct>,
    depth: usize,
) -> Result<TokenStream2> {
    let name = &s.name;
    let field_names: Vec<_> = s.fields.iter().map(|f| &f.name).collect();
    let field_values = s
        .fields
        .iter()
        .map(|field| expand_field(field, module, registry, depth))
        .collect::<Result<Vec<_>>>()?;

    Ok(quote::quote! {
        #name {
            #( #field_names: #field_values, )*
        }
    })
}

fn expand_field(
    field: &Field,
    module: &Option<TokenStream2>,
    registry: &HashMap<String, &Struct>,
    depth: usize,
) -> Result<TokenStream2> {
    match &field.value {
        FieldValue::Leaf(leaf) => Ok(expand_leaf(field, leaf, module)),

        FieldValue::Reference(ty) => {
            let target = resolve_struct(ty, registry)?;
            let child = compose_module(module, &field.name, None);
            expand_struct_value(target, &Some(child), registry, depth)
        }

        FieldValue::Array(ArrayField {
            element: ArrayElement::Named(ty),
            ..
        }) => {
            let target = resolve_struct(ty, registry)?;
            let index = array_index(depth);
            let child = compose_module(module, &field.name, Some(&index));
            let value = expand_struct_value(target, &Some(child), registry, depth + 1)?;
            Ok(quote::quote! { ::core::array::from_fn(|#index| #value) })
        }

        FieldValue::Array(ArrayField {
            element: ArrayElement::Inline(_),
            ..
        }) => unreachable!("anonymous `Array` element is hoisted during collect"),
        FieldValue::Inline(_) => unreachable!("anonymous group is hoisted during collect"),
    }
}

/// `ParamType::create(ClapId::new(cursor), "name", Some(module), Config { .. })`.
fn expand_leaf(field: &Field, leaf: &Leaf, module: &Option<TokenStream2>) -> TokenStream2 {
    let Leaf { ty, entries, .. } = leaf;
    let param = type_path_expr(ty);
    let config = type_path_expr(&config_type(ty));
    let name = syn::LitStr::new(&field.name.to_string(), field.name.span());
    let module = match module {
        Some(path) => quote::quote! { Some(#path) },
        None => quote::quote! { None },
    };
    let keys: Vec<_> = entries.iter().map(|e| &e.key).collect();
    let values: Vec<_> = entries.iter().map(|e| &e.value).collect();

    quote::quote! {
        {
            let id = ::rift_plugin::prelude::clack_plugin::prelude::ClapId::new(cursor);
            cursor += 1u32;
            #param::create(id, #name.to_string(), #module, #config {
                #( #keys: #values, )*
                ..Default::default()
            })
        }
    }
}

/// Build the `module` string expression for a child of `parent`.
fn compose_module(
    parent: &Option<TokenStream2>,
    field: &Ident,
    index: Option<&Ident>,
) -> TokenStream2 {
    let field = syn::LitStr::new(&field.to_string(), field.span());
    match (parent, index) {
        (None, None) => quote::quote! { #field.to_string() },
        (None, Some(index)) => quote::quote! { format!("{}[{}]", #field, #index) },
        (Some(parent), None) => quote::quote! { format!("{}.{}", #parent, #field) },
        (Some(parent), Some(index)) => {
            quote::quote! { format!("{}.{}[{}]", #parent, #field, #index) }
        }
    }
}

/// Index variable name for the array nested at `depth` (`i0`, `i1`, ...).
fn array_index(depth: usize) -> Ident {
    Ident::new(&format!("i{depth}"), Span::call_site())
}

fn resolve_struct<'a>(ty: &Type, registry: &'a HashMap<String, &'a Struct>) -> Result<&'a Struct> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(ty, "expected a struct type"));
    };
    let segment = type_path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(ty, "expected a struct type"))?;
    let key = segment.ident.to_string();

    registry.get(&key).copied().ok_or_else(|| {
        syn::Error::new_spanned(
            ty,
            format!("unknown struct `{key}`; it must be defined in this `params!` block"),
        )
    })
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

/// The Rust type of a generated struct field.
fn field_type(field: &Field) -> TokenStream2 {
    match &field.value {
        FieldValue::Leaf(Leaf { kind, ty, .. }) => match kind {
            LeafKind::Param => quote::quote! { #ty },
        },
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
