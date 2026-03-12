use proc_macro::TokenStream;

mod derive_extensions;
mod derive_param;
mod enum_param;
mod param_builder;

/// Generate boiler plate code for the builder > destructure > build view pattern.
///
/// This is meant to be used with `DestructThenBuildView` trait.
///
/// # Example:
/// ```ignore
/// #[derive(ParamViewBuilder)]
/// struct MyParamView {
///     // this value must be passed to contructor (no need for default)
///     #[builder(new)]
///     value1: f32
///     
///     // Can be added with builder pattern (needs to impl Default or have a default specified)
///     #[builder(default = 10.)]
///     value2: f32
/// }
///
/// impl DestructThenBuildView for MyParamView {
///     fn build_view(self, cx: &mut Context) -> Handle<'_, impl View> { ... }
/// }
///
/// MyParamView::new(1.0).value2(5.0).build_view(cx);
/// ```
#[proc_macro_derive(ParamViewBuilder, attributes(builder))]
pub fn derive_param_builder(input: TokenStream) -> TokenStream {
    param_builder::derive_param_builder(input)
}

#[proc_macro_derive(DeriveEnumValues, attributes(enum_values))]
pub fn derive_enum_values(input: TokenStream) -> TokenStream {
    enum_param::derive_enum_values(input)
}

#[proc_macro_derive(DeriveParams, attributes(param))]
pub fn derive_params(input: TokenStream) -> TokenStream {
    derive_param::derive_params(input)
}

#[proc_macro_derive(HandleExtension, attributes(extension))]
pub fn derive_extensions(input: TokenStream) -> TokenStream {
    derive_extensions::derive_extensions(input)
}
