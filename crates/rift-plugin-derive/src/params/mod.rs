//! The `params!` procedural macro.
//!
//! The pipeline has two stages, each in its own module:
//!
//! - [`collect`] parses the token stream and *resolves* it into a
//!   self-contained IR (structs hoisted and named, references substituted,
//!   module paths and ids computed).
//! - [`expand`] renders that IR into Rust code, without any further lookup.

mod collect;
mod expand;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

pub fn proc_params(input: TokenStream) -> TokenStream {
    match syn::parse::<collect::Params>(input)
        .and_then(collect::Params::resolve)
        .map(expand::expand)
    {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
