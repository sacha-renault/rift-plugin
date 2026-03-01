use proc_macro::TokenStream;

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
