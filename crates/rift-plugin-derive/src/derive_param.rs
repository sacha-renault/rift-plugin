use std::collections::HashSet;

// In your proc-macro crate
use darling::{FromDeriveInput, FromField, ast};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[derive(FromField)]
#[darling(attributes(param))]
struct ParamField {
    ident: Option<syn::Ident>,

    #[allow(unused)]
    ty: syn::Type,
    // /// If not present, it defaults to None.
    // #[darling(default)]
    // default: Option<syn::Expr>,
    #[darling(default)]
    name: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(param), supports(struct_named))]
struct ParamsReceiver {
    ident: syn::Ident,
    data: ast::Data<(), ParamField>,
}

pub fn derive_params(input: TokenStream) -> TokenStream {
    // parse
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = ParamsReceiver::from_derive_input(&input).unwrap();
    let name = &receiver.ident;
    let fields = receiver.data.take_struct().expect("Expected struct");

    // general info (count etc ..)
    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let count = field_idents.len() as u32;
    let indices: Vec<u32> = (0..count).collect();

    let mut unique_names = HashSet::<String>::new();
    let param_names: Vec<_> = fields
        .iter()
        .map(|f| {
            if let Some(name) = f.name.clone() {
                if unique_names.contains(&name) {
                    panic!("Parameter name `{}` is not unique", name)
                }
                unique_names.insert(name.clone());
                name
            } else {
                let ident = f.ident.as_ref().unwrap().to_string();
                panic!("Parameter {} must have a `name`.", ident);
            }
        })
        .collect();

    let expanded = quote! {
        impl ::rift_plugin::prelude::Params for #name {
            fn count(&self) -> u32 {
                #count
            }

            fn get_param_info<'a>(&'a self, index: u32) -> Option<::rift_plugin::prelude::clack_extensions::params::ParamInfo<'a>> {
                match index {
                    #(
                        #indices => Some(self.#field_idents.param_info()),
                    )*
                    _ => None,
                }
            }

            fn get_value(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId) -> Option<f64> {
                #(
                    if id == self.#field_idents.id() {
                        return Some(self.#field_idents.get_raw());
                    }
                )*
                None
            }

            fn set_value(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId, value: f64) {
                #(
                    if id == self.#field_idents.id() {
                        self.#field_idents.set_raw(value);
                        return;
                    }
                )*
            }

            fn set_value_normalized(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId, value: f64) {
                #(
                    if id == self.#field_idents.id() {
                        self.#field_idents.set_normalized(value);
                        return;
                    }
                )*
            }

            fn text_to_value(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId, text: &std::ffi::CStr) -> Option<f64> {
                #(
                    if id == self.#field_idents.id() {
                        return self.#field_idents.text_to_value(text);
                    }
                )*
                None
            }

            fn value_to_text(
                &self,
                id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId,
                value: f64,
                writer: &mut ::rift_plugin::prelude::clack_extensions::params::ParamDisplayWriter,
            ) -> std::fmt::Result {
                #(
                    if id == self.#field_idents.id() {
                        return self.#field_idents.value_to_text(value, writer);
                    }
                )*
                Err(std::fmt::Error)
            }
        }

        impl ::rift_plugin::_sealed::__ParamsInitializer for #name {
            fn __initialize(&mut self) {
                use ::rift_plugin::_sealed::__ParamInitializer;
                #(
                    self.#field_idents.__initialize(
                        #param_names.to_string(),
                        ::rift_plugin::prelude::clack_plugin::prelude::ClapId::from(#indices),
                        None,
                    );
                )*
            }
        }
    };

    TokenStream::from(expanded)
}
