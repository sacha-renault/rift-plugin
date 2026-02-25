use proc_macro::TokenStream;

mod param_builder;

#[proc_macro_derive(ParamBuilder, attributes(builder))]
pub fn derive_param_builder(input: TokenStream) -> TokenStream {
    param_builder::derive_param_builder(input)
}
