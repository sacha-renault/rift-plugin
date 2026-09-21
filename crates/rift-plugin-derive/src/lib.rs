use proc_macro::TokenStream;

mod enum_param;
mod params;

#[proc_macro_derive(DeriveEnumValues, attributes(enum_values))]
pub fn derive_enum_values(input: TokenStream) -> TokenStream {
    enum_param::derive_enum_values(input)
}

#[proc_macro]
pub fn params(input: TokenStream) -> TokenStream {
    params::proc_params(input)
}
