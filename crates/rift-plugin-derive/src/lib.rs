use proc_macro::TokenStream;

mod enum_param;
#[cfg(feature = "modifiers")]
mod modifier;
mod params;

#[proc_macro_derive(DeriveEnumValues, attributes(enum_values))]
pub fn derive_enum_values(input: TokenStream) -> TokenStream {
    enum_param::derive_enum_values(input)
}

#[proc_macro_derive(Params, attributes(param, nested))]
pub fn derive_params(input: TokenStream) -> TokenStream {
    params::derive_params(input)
}

/// Emits the concrete impl for an extension trait alongside the trait itself.
///
/// Only available with the `modifiers` feature. See the `modifier` module for
/// the shape and a worked example.
#[cfg(feature = "modifiers")]
#[proc_macro_attribute]
pub fn modifiers(attr: TokenStream, item: TokenStream) -> TokenStream {
    modifier::modifiers(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
