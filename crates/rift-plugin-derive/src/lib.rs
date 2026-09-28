use proc_macro::TokenStream;

mod enum_param;
mod params;

#[proc_macro_derive(DeriveEnumValues, attributes(enum_values))]
pub fn derive_enum_values(input: TokenStream) -> TokenStream {
    enum_param::derive_enum_values(input)
}

#[proc_macro_derive(Params, attributes(param, nested))]
pub fn derive_params(input: TokenStream) -> TokenStream {
    params::derive_params(input)
}
