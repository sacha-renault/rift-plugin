// In your proc-macro crate
use darling::{FromDeriveInput, FromField, ast};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::param_builder::inner_type_of_option;

#[derive(Debug, FromField)]
#[darling(attributes(extension))]
struct ParamField {
    ident: Option<syn::Ident>,

    #[allow(unused)]
    ty: syn::Type,

    #[darling(default)]
    ext: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(extension), supports(struct_named))]
struct ParamsReceiver {
    ident: syn::Ident,
    data: ast::Data<(), ParamField>,
}

pub fn derive_extensions(input: TokenStream) -> TokenStream {
    // parse
    let input = parse_macro_input!(input as DeriveInput);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let receiver = ParamsReceiver::from_derive_input(&input).unwrap();

    let name = &receiver.ident;
    let fields = receiver.data.take_struct().expect("Expected struct");
    let extensions_field: Vec<_> = fields.iter().filter(|field| field.ext).collect();

    if extensions_field.is_empty() {
        return quote! {}.into();
    }

    let extension_name = syn::Ident::new(&format!("{}Ext", name), Span::call_site());
    let setters: Vec<_> = extensions_field
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let ty = &field.ty;

            if let Some(peeled_option_type) = inner_type_of_option(ty) {
                quote! { fn #field_name (self, value: #peeled_option_type) -> Self {
                    self.modify(|view| view.#field_name = Some(value))
                } }
            } else {
                quote! { fn #field_name (self, value: #ty) -> Self {
                    self.modify(|view| view.#field_name = value)
                } }
            }
        })
        .collect();

    let defs: Vec<_> = extensions_field
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let ty = &field.ty;

            if let Some(peeled_option_type) = inner_type_of_option(ty) {
                quote! { fn #field_name (self, value: #peeled_option_type) -> Self; }
            } else {
                quote! { fn #field_name (self, value: #ty) -> Self; }
            }
        })
        .collect();

    quote! {
        pub trait #extension_name {
            #(#defs)*
        }

        impl #impl_generics #extension_name for Handle<'_, #name > #ty_generics #where_clause {
            #(#setters)*
        }
    }
    .into()
}
