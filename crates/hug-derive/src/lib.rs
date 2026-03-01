use proc_macro::TokenStream;

mod derive_param;
mod enum_param;
mod param_builder;

#[proc_macro_derive(ParamViewBuilder, attributes(builder))]
pub fn derive_param_builder(input: TokenStream) -> TokenStream {
    param_builder::derive_param_builder(input)
}

#[proc_macro_derive(DeriveEnumValues, attributes(enum_values))]
pub fn derive_enum_values(input: TokenStream) -> TokenStream {
    enum_param::derive_enum_values(input)
}

#[proc_macro_derive(DeriveParams, attributes(params))]
pub fn derive_params(input: TokenStream) -> TokenStream {
    derive_param::derive_params(input)
}
