use std::collections::HashSet;

// In your proc-macro crate
use darling::{FromDeriveInput, FromField, ast};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[derive(FromField)]
#[darling(attributes(param))]
struct ParamField {
    ident: Option<syn::Ident>,

    #[allow(unused)]
    ty: syn::Type,

    #[darling(default)]
    name: Option<String>, // default to None

    #[darling(default)]
    persistent: bool, // default to false
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
    let mut unique_names = HashSet::<String>::new();

    // Separate fields into param fields and persistent-only fields
    let mut param_fields_idents = Vec::new();
    let mut param_fields_names = Vec::new();
    let mut param_fields_types = Vec::new();

    let mut persist_fields_idents = Vec::new();
    let mut persist_fields_names = Vec::new();
    let mut persist_fields_types = Vec::new();

    for f in fields.iter() {
        let ident = f.ident.as_ref().unwrap();
        let field_name = if let Some(ref name) = f.name {
            if unique_names.contains(name) {
                panic!("Parameter name `{name}` is not unique");
            }
            unique_names.insert(name.clone());
            name.clone()
        } else {
            panic!("Parameter {} must have a `name`.", ident);
        };

        if f.persistent {
            persist_fields_idents.push(ident);
            persist_fields_names.push(field_name);
            persist_fields_types.push(&f.ty);
        } else {
            param_fields_idents.push(ident);
            param_fields_names.push(field_name);
            param_fields_types.push(&f.ty);
        }
    }

    let param_count = param_fields_idents.len() as u32;
    let param_indices: Vec<u32> = (0..param_count).collect();
    let param_name_id: Vec<_> = param_fields_names
        .iter()
        .map(|name| name.replace(" ", "_") + "_ID")
        .map(|name| syn::Ident::new(&name, Span::call_site()))
        .collect();
    let persist_name_id: Vec<_> = persist_fields_names
        .iter()
        .map(|name| name.replace(" ", "_") + "_ID")
        .map(|name| syn::Ident::new(&name, Span::call_site()))
        .collect();

    // Persistent fields get indices after param fields (for __initialize ClapId)
    let persist_indices: Vec<u32> =
        (param_count..(param_count + persist_fields_idents.len() as u32)).collect();

    // All field types for the Persistent trait check
    let all_field_types: Vec<_> = param_fields_types
        .iter()
        .chain(persist_fields_types.iter())
        .collect();

    let expanded = quote! {
        impl ::rift_plugin::prelude::Params for #name {
            fn count(&self) -> u32 {
                #param_count
            }

            fn get_param_info<'a>(&'a self, index: u32) -> Option<::rift_plugin::prelude::clack_extensions::params::ParamInfo<'a>> {
                match index {
                    #(
                        #param_indices => Some(self.#param_fields_idents.param_info()),
                    )*
                    _ => None,
                }
            }

            fn get_value(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId) -> Option<f32> {
                #(
                    if id == self.#param_fields_idents.id() {
                        return Some(self.#param_fields_idents.get_raw());
                    }
                )*
                None
            }

            fn set_value(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId, value: f32) {
                #(
                    if id == self.#param_fields_idents.id() {
                        self.#param_fields_idents.set_raw(value);
                        return;
                    }
                )*
            }

            fn set_value_normalized(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId, value: f32) {
                #(
                    if id == self.#param_fields_idents.id() {
                        self.#param_fields_idents.set_normalized(value);
                        return;
                    }
                )*
            }

            fn text_to_value(&self, id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId, text: &std::ffi::CStr) -> Option<f32> {
                #(
                    if id == self.#param_fields_idents.id() {
                        return self.#param_fields_idents.text_to_value(text);
                    }
                )*
                None
            }

            fn value_to_text(
                &self,
                id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId,
                value: f32,
                writer: &mut ::rift_plugin::prelude::clack_extensions::params::ParamDisplayWriter,
            ) -> std::fmt::Result {
                #(
                    if id == self.#param_fields_idents.id() {
                        return self.#param_fields_idents.value_to_text(value as f32, writer);
                    }
                )*
                Err(std::fmt::Error)
            }

            fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<(), ::rift_plugin::prelude::PluginError> {
                use std::io::Write;
                use rift_plugin::_sealed::serde_json;

                let mut map = serde_json::Map::new();
                // Serialize param fields
                #(
                    {
                        let mut buf = Vec::new();
                        self.#param_fields_idents.serialize(&mut buf)?;
                        let value: serde_json::Value = serde_json::from_slice(&buf)
                            .map_err(|_| ::rift_plugin::prelude::PluginError::Message("serialize error"))?;
                        map.insert(#param_fields_names.to_string(), value);
                    }
                )*
                // Serialize persistent-only fields
                #(
                    {
                        let mut buf = Vec::new();
                        self.#persist_fields_idents.serialize(&mut buf)?;
                        let value: serde_json::Value = serde_json::from_slice(&buf)
                            .map_err(|_| ::rift_plugin::prelude::PluginError::Message("serialize error"))?;
                        map.insert(#persist_fields_names.to_string(), value);
                    }
                )*
                serde_json::to_writer(writer, &map)
                    .map_err(|_| ::rift_plugin::prelude::PluginError::Message("serialize error"))
            }

            fn deserialize(&self, reader: &mut dyn std::io::Read) -> Result<(), ::rift_plugin::prelude::PluginError> {
                use rift_plugin::_sealed::serde_json;

                let map: serde_json::Map<String, serde_json::Value> = serde_json::from_reader(reader)
                    .map_err(|_| ::rift_plugin::prelude::PluginError::Message("deserialize error"))?;
                // Deserialize param fields
                #(
                    if let Some(value) = map.get(#param_fields_names) {
                        let buf = serde_json::to_vec(value)
                            .map_err(|_| ::rift_plugin::prelude::PluginError::Message("deserialize error"))?;
                        self.#param_fields_idents.deserialize(&mut buf.as_slice())?;
                    }
                )*
                // Deserialize persistent-only fields
                #(
                    if let Some(value) = map.get(#persist_fields_names) {
                        let buf = serde_json::to_vec(value)
                            .map_err(|_| ::rift_plugin::prelude::PluginError::Message("deserialize error"))?;
                        self.#persist_fields_idents.deserialize(&mut buf.as_slice())?;
                    }
                )*
                Ok(())
            }
        }

        impl ::rift_plugin::_sealed::__ParamsInitializer for #name {
            fn __initialize(&mut self) {
                use ::rift_plugin::_sealed::__ParamInitializer;
                // Initialize param fields
                #(
                    self.#param_fields_idents.__initialize(
                        #param_fields_names.to_string(),
                        ::rift_plugin::prelude::clack_plugin::prelude::ClapId::new(#param_indices),
                        None,
                    );
                )*
                // Initialize persistent-only fields
                #(
                    self.#persist_fields_idents.__initialize(
                        #persist_fields_names.to_string(),
                        ::rift_plugin::prelude::clack_plugin::prelude::ClapId::new(#persist_indices),
                        None,
                    );
                )*
            }
        }

        // We ensure params DOES implement Persistant trait
        const _: () = {
            fn _assert_persistent<T: ::rift_plugin::prelude::Persistent>() { }
            fn _assert_clap_param<T: ::rift_plugin::prelude::ClapParam>() { }
            fn _check() {
                #( _assert_persistent::<#all_field_types>(); )*
                #( _assert_clap_param::<#param_fields_types>(); )*
            }
        };

        pub mod param_ids {
            #(
                pub const #param_name_id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId
                    = ::rift_plugin::prelude::clack_plugin::prelude::ClapId::new(#param_indices);
            )*

            #(
                pub const #persist_name_id: ::rift_plugin::prelude::clack_plugin::prelude::ClapId
                    = ::rift_plugin::prelude::clack_plugin::prelude::ClapId::new(#persist_indices);
            )*
        }
    };

    TokenStream::from(expanded)
}
