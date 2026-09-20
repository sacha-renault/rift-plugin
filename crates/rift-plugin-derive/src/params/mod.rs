//! The `params!` procedural macro.
//!
//! - [`collect`] parses the token stream into an AST and collects the structs.
//! - [`expand`] turns the collected structs into Rust code (type definitions and
//!   the root `Parameters::create()`).

mod collect;
mod expand;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

pub fn proc_params(input: TokenStream) -> TokenStream {
    match syn::parse::<collect::Params>(input).and_then(|params| params.expand()) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
