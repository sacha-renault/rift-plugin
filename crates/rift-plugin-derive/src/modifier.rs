//! the point of this macro is to make easier
//! blanket implementation for foreign type
//!
//! This is particulary useful in vizia because
//! we Handle<'_, V> is a foreign type and we might
//! want to add extensions.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, GenericParam, Generics, Ident, ItemTrait, Token, TraitItem, TypeParam, WhereClause,
    WherePredicate,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Arguments of `#[modifiers(for <Type> [where ...])]`.
struct Args {
    self_ty: syn::Type,
    generics: Generics,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _: Token![for] = input.parse()?;
        let self_ty = input.parse()?;

        let where_clause: Option<WhereClause> = if input.peek(Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        if !input.is_empty() {
            return Err(input.error("expected `for <Type>` optionally followed by `where ...`"));
        }

        Ok(Self {
            self_ty,
            generics: generics_from(&where_clause),
        })
    }
}

/// Derives the impl's generic parameters from the free types bound by the
/// user-supplied optional where clause.
fn generics_from(where_clause: &Option<WhereClause>) -> Generics {
    let Some(where_clause) = where_clause else {
        return Generics::default();
    };

    let mut params: Punctuated<GenericParam, Token![,]> = Punctuated::new();
    let mut seen: Vec<syn::Ident> = Vec::new();
    for predicate in &where_clause.predicates {
        let WherePredicate::Type(predicate) = predicate else {
            continue;
        };
        let syn::Type::Path(type_path) = &predicate.bounded_ty else {
            continue;
        };
        if type_path.qself.is_some() || type_path.path.segments.len() != 1 {
            continue;
        }
        let Some(segment) = type_path.path.segments.first() else {
            continue;
        };
        if !matches!(segment.arguments, syn::PathArguments::None) || seen.contains(&segment.ident) {
            continue;
        }
        seen.push(segment.ident.clone());
        params.push(GenericParam::Type(TypeParam {
            attrs: Vec::new(),
            ident: segment.ident.clone(),
            colon_token: None,
            bounds: Punctuated::new(),
            eq_token: None,
            default: None,
        }));
    }

    Generics {
        where_clause: Some(where_clause.clone()),
        params,
        ..Default::default()
    }
}

pub(crate) fn modifiers(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let Args { self_ty, generics } = syn::parse2(attr)?;
    let mut trait_def: ItemTrait = syn::parse2(item)?;

    // Methods split out of the trait into the concrete impl.
    let mut concrete: Vec<syn::ImplItemFn> = Vec::new();

    for item in &mut trait_def.items {
        let TraitItem::Fn(method) = item else {
            continue;
        };

        if remove_attr_by_ident(&mut method.attrs, "concrete").is_none() {
            continue;
        }

        let Some(block) = method.default.take() else {
            return Err(syn::Error::new_spanned(
                &method.sig,
                "#[concrete] methods must have a body: it becomes the impl's body",
            ));
        };

        // The body now lives in the impl, so the trait method becomes a bare
        // signature terminated by a `;`.
        method.semi_token = Some(Default::default());

        // Clone now attrs
        let concrete_attrs = method.attrs.clone();

        // Some path items shouldn't exist on trait
        remove_attr_by_ident(&mut method.attrs, "inline");

        concrete.push(syn::ImplItemFn {
            attrs: concrete_attrs,
            vis: syn::Visibility::Inherited,
            defaultness: None,
            sig: method.sig.clone(),
            block,
        });
    }

    let trait_ident = &trait_def.ident;
    let where_clause = &generics.where_clause;
    let impl_block = if concrete.is_empty() {
        TokenStream::new()
    } else {
        quote! {
            impl #generics #trait_ident for #self_ty #where_clause {
                #(#concrete)*
            }
        }
    };

    Ok(quote! {
        #trait_def
        #impl_block
    })
}

fn remove_attr_by_ident<I>(attrs: &mut Vec<Attribute>, ident: &I) -> Option<Attribute>
where
    I: ?Sized,
    Ident: PartialEq<I>,
{
    if let Some(idx) = attrs.iter().position(|attr| attr.path().is_ident(ident)) {
        Some(attrs.remove(idx))
    } else {
        None
    }
}
