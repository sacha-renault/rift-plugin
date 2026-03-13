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

    #[darling(default)]
    with: Option<syn::Expr>,

    #[darling(default)]
    set: Option<syn::Expr>,

    #[darling(default)]
    setter_name: Option<syn::Ident>,

    #[darling(default)]
    call_method: Option<syn::Expr>,

    #[darling(default)]
    arg_ty: Option<syn::Expr>,
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
            let signature = define_signature(field);
            let body = define_body(field);

            quote! { #signature {
                #body
            }}
        })
        .collect();

    let defs: Vec<_> = extensions_field
        .iter()
        .map(|field| {
            let signature = define_signature(field);
            quote! { #signature; }
        })
        .collect();

    quote! {
        pub trait #extension_name {
            #(#defs)*
        }

        impl #impl_generics #extension_name for Handle<'_, #name #ty_generics>  #where_clause {
            #(#setters)*
        }
    }
    .into()
}

/// Define the signature of the builder like function
fn define_signature(field: &ParamField) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let ty = &field.ty;
    let fn_name = field
        .setter_name
        .as_ref()
        .map(|name| quote! { #name })
        .unwrap_or(quote! { #field_name });

    // SET, in case set is defined, the function takes no input and just set
    // to the defined value
    if field.set.is_some() {
        quote! { fn #fn_name (self) -> Self }
    } else if let Some(override_type) = &field.arg_ty {
        quote! { fn #fn_name (self, value: #override_type) -> Self }
    } else if let Some(peeled_option_type) = inner_type_of_option(ty) {
        quote! { fn #fn_name (self, value: #peeled_option_type) -> Self }
    } else {
        quote! { fn #fn_name (self, value: #ty) -> Self }
    }
}

/// Define the body of the builder like function
fn define_body(field: &ParamField) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let ty = &field.ty;

    let mut value = quote! { value };

    if let Some(const_value) = &field.set {
        // Override, we don't take value from
        // the fn argument, we have our own defined
        // value, constant for the struct
        value = quote! { #const_value };
    }

    if let Some(with) = &field.with {
        value = quote! { (#with)(#value) }
    }

    if inner_type_of_option(ty).is_some() {
        value = quote! { Some(#value) };
    }

    if let Some(method) = &field.call_method {
        quote! { self.modify(|view| view.#field_name.#method (#value) ) }
    } else {
        quote! { self.modify(|view| view.#field_name = #value ) }
    }
}
