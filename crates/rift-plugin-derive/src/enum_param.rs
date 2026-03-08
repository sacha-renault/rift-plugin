use darling::{FromDeriveInput, FromVariant, ast};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[derive(FromVariant)]
#[darling(attributes(enum_values))]
struct VariantReceiver {
    ident: syn::Ident,
    fields: ast::Fields<()>,

    #[darling(default)]
    text: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(enum_values), supports(enum_unit))]
struct EnumReceiver {
    ident: syn::Ident,
    data: ast::Data<VariantReceiver, ()>,
}

pub fn derive_enum_values(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let receiver = EnumReceiver::from_derive_input(&input).unwrap();
    let name = &receiver.ident;
    let variants = receiver.data.take_enum().expect("Expected enum");

    // Validate all variants are unit variants
    for v in &variants {
        if !v.fields.is_empty() {
            panic!("EnumValues only supports unit variants");
        }
    }

    let variant_names: Vec<_> = variants.iter().map(|v| &v.ident).collect();
    let variant_texts: Vec<_> = variants
        .iter()
        .map(|v| match &v.text {
            Some(t) => quote! { #t },
            None => {
                let ident = &v.ident;
                quote! { stringify!(#ident) }
            }
        })
        .collect();

    let count = variant_names.len() as u32;
    let indices: Vec<u32> = (0..count).collect();

    let expanded = quote! {
        impl ::clack_hug::prelude::EnumValues for #name {
            fn to_index(self) -> u32 {
                self as u32
            }

            fn from_index(index: u32) -> Option<Self> {
                match index {
                    #( #indices => Some(Self::#variant_names), )*
                    _ => None,
                }
            }

            fn count() -> u32 {
                #count
            }
        }

        impl Copy for #name {}
        impl Clone for #name {
            fn clone(&self) -> Self { *self }
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let text = match self {
                    #(Self::#variant_names => #variant_texts, )*
                };
                write!(f, "{}", text)
            }
        }
    };

    TokenStream::from(expanded)
}
