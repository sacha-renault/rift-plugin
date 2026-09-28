//! The `Params` derive macro.
//!
//! [`parse`] turns the annotated struct into a small IR, [`expand`] renders it
//! into the constructor (`new`/`Default`) and the `UserParams` implementation.

mod expand;
mod parse;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;
use syn::parse_macro_input;

pub fn derive_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    match parse::Params::from_derive_input(&input) {
        Ok(params) => expand::expand(params).into(),
        Err(error) => error.into_compile_error().into(),
    }
}
