//! the point of this macro is to make easier
//! blanket implementation for foreign type
//!
//! This is particulary useful in vizia because
//! we Handle<'_, V> is a foreign type and we might
//! want to add extensions.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemTrait, Token, TraitItem, TraitItemFn, parse::Parse, parse::ParseStream};

/// Arguments of `#[modifiers(for <Type>)]`.
struct Args {
    self_ty: syn::Type,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: Token![for] = input.parse()?;
        let self_ty = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("expected `for <Type>`"));
        }

        Ok(Self { self_ty })
    }
}

pub(crate) fn modifiers(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let Args { self_ty } = syn::parse2(attr)?;
    let mut trait_def: ItemTrait = syn::parse2(item)?;

    // Methods split out of the trait into the concrete impl.
    let mut concrete: Vec<syn::ImplItemFn> = Vec::new();

    for item in &mut trait_def.items {
        let TraitItem::Fn(method) = item else {
            continue;
        };

        if !is_concrete(method) {
            continue;
        }

        // The marker is a helper only; it must not reach the compiler.
        method
            .attrs
            .retain(|attr| !attr.path().is_ident("concrete"));

        let Some(block) = method.default.take() else {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "#[concrete] methods must have a body: it becomes the impl's body",
            ));
        };

        // The body now lives in the impl, so the trait method becomes a bare
        // signature terminated by a `;`.
        method.semi_token = Some(Default::default());

        concrete.push(syn::ImplItemFn {
            attrs: method.attrs.clone(),
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: method.sig.clone(),
            block,
        });
    }

    let trait_ident = &trait_def.ident;
    let impl_block = if concrete.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            impl #trait_ident for #self_ty {
                #(#concrete)*
            }
        }
    };

    Ok(quote! {
        #trait_def
        #impl_block
    })
}

fn is_concrete(method: &TraitItemFn) -> bool {
    method
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("concrete"))
}
