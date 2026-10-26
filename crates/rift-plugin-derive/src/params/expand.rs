//! Rendering of the parsed `#[derive(Params)]` struct into Rust code.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Expr, ExprLit, ExprUnary, Lit, LitFloat, LitStr, PathArguments, Type, UnOp};

use super::parse::{Field, LeafParam, LeafPersist, Nested, ParamKind, Params, RangeSpec, Skipped};

/// Entry point: render the `new`/`Default`/`UserParams` implementations.
pub(crate) fn expand(params: Params) -> TokenStream2 {
    let Params {
        ident,
        generics,
        fields,
    } = params;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let construct = expand_construct(&fields);
    let all_params = expand_all_params(&fields);
    let all_persist = expand_all_persist(&fields);

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            /// Builds the parameter tree with fully resolved ids, names and modules.
            pub fn new() -> Self {
                Self::create_with_module(::core::option::Option::None)
            }

            #[doc(hidden)]
            pub fn create_with_module(
                __module: ::core::option::Option<::std::string::String>,
            ) -> Self {
                #construct
            }
        }

        impl #impl_generics ::core::default::Default for #ident #ty_generics #where_clause {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #impl_generics ::rift_plugin::prelude::UserParams for #ident #ty_generics #where_clause {
            fn all_params(&self) -> ::std::vec::Vec<::rift_plugin::prelude::ParamPtr> {
                let mut __params =
                    ::std::vec::Vec::<::rift_plugin::prelude::ParamPtr>::new();
                #all_params
                __params
            }

            fn all_persist(&self) -> ::std::vec::Vec<&dyn ::rift_plugin::prelude::Persistent> {
                let mut __persist =
                    ::std::vec::Vec::<&dyn ::rift_plugin::prelude::Persistent>::new();
                #all_persist
                __persist
            }
        }
    }
}

/// The `Self { .. }` body of `create_with_module`.
fn expand_construct(fields: &[Field]) -> TokenStream2 {
    let inits = fields.iter().map(|field| {
        let ident = match field {
            Field::Param(leaf) => &leaf.ident,
            Field::Persist(leaf) => &leaf.ident,
            Field::Nested(nested) => &nested.ident,
            Field::Skip(skip) => &skip.ident,
        };
        let value = match field {
            Field::Param(leaf) => expand_leaf(leaf),
            Field::Persist(leaf) => expand_persist(leaf),
            Field::Nested(nested) => expand_nested(nested),
            Field::Skip(skip) => expand_skip(skip),
        };
        quote! { #ident: #value, }
    });

    quote! { Self { #( #inits )* } }
}

/// The construction of a single leaf parameter: `Ty::builder()..build()`.
fn expand_leaf(leaf: &LeafParam) -> TokenStream2 {
    let ty = type_path_expr(&leaf.ty);

    let field = leaf.ident.to_string();
    let name = leaf.name.clone().unwrap_or_else(|| field.clone());
    // The id defaults to the *field* identifier, not the display name, so that
    // renaming a label does not silently change the id (and thus saved state).
    let id = leaf.resolve_string_id();

    let name_lit = LitStr::new(&name, leaf.ident.span());
    let id_lit = LitStr::new(&id, leaf.ident.span());

    let default = leaf.default.as_ref().map(|expr| {
        let expr = coerce_float(expr, leaf.kind);
        quote! { .default(#expr) }
    });

    let range = leaf.range.as_ref().map(|range| expand_range(range, leaf));

    let unit = leaf.unit.as_ref().map(|unit| {
        let lit = LitStr::new(unit, leaf.ident.span());
        quote! { .unit(#lit) }
    });
    let flags = leaf.flags.as_ref().map(|expr| quote! { .flags(#expr) });
    let tags = leaf.tags.as_ref().map(|expr| quote! { .tags(#expr) });

    quote! {
        <#ty>::builder()
            .id(::rift_plugin::prelude::param_id(
                __module.as_deref().unwrap_or(""),
                #id_lit
            ))
            .name(::std::string::String::from(#name_lit))
            .maybe_module(__module.clone())
            #default
            #range
            #unit
            #flags
            #tags
            .build()
    }
}

/// The construction of a single `#[persist]` field.
fn expand_persist(leaf: &LeafPersist) -> TokenStream2 {
    let ty = type_path_expr(&leaf.ty);

    let name = leaf.name.clone().unwrap_or_else(|| leaf.ident.to_string());
    let id = leaf.resolve_string_id();
    let name_lit = LitStr::new(&name, leaf.ident.span());
    let id_lit = LitStr::new(&id, leaf.ident.span());

    let tags = match leaf.tags.as_ref() {
        Some(expr) => quote! { #expr },
        None => quote! { ::rift_plugin::prelude::Tags::default() },
    };

    quote! {
        <#ty as ::rift_plugin::prelude::Persistent>::create(
            ::rift_plugin::prelude::param_id(
                __module.as_deref().unwrap_or(""),
                #id_lit
            ),
            ::std::string::String::from(#name_lit),
            __module.clone(),
            #tags,
        )
    }
}

/// The construction of a `#[skip]` field: its explicit default when given, and
/// `Default::default()` otherwise.
fn expand_skip(skip: &Skipped) -> TokenStream2 {
    match skip.default.as_ref() {
        Some(expr) => quote! { #expr },
        None => quote! { ::core::default::Default::default() },
    }
}

/// Render a `range = ...` into the kind-specific constructor call.
///
/// The call is wrapped in a `const` block so the constructor's asserts run at
/// compile time.
fn expand_range(range: &RangeSpec, leaf: &LeafParam) -> TokenStream2 {
    let range_ty = match leaf.kind {
        ParamKind::Float => quote!(::rift_plugin::prelude::FloatRange),
        ParamKind::Int => quote!(::rift_plugin::prelude::IntRange),
        // Reject range for any other kind
        ParamKind::Bool | ParamKind::Enum => unreachable!("range on a kind without one"),
    };

    let ctor = &range.ctor;
    let args = range.args.iter().map(|arg| coerce_float(arg, leaf.kind));

    quote! { .range(const { #range_ty::#ctor(#(#args),*) }) }
}

/// The construction of a nested struct (or array of structs), threading the
/// child module path down to the nested struct.
fn expand_nested(nested: &Nested) -> TokenStream2 {
    let segment_str = nested
        .module
        .clone()
        .unwrap_or_else(|| nested.ident.to_string());
    let segment = LitStr::new(&segment_str, nested.ident.span());
    let elem = &nested.element_ty;

    if nested.array_len.is_some() {
        quote! {
            ::core::array::from_fn(|__i| {
                let __child_module = match __module.as_deref() {
                    ::core::option::Option::Some(__parent) => {
                        ::std::format!("{}.{}[{}]", __parent, #segment, __i)
                    }
                    ::core::option::Option::None => ::std::format!("{}[{}]", #segment, __i),
                };
                <#elem>::create_with_module(::core::option::Option::Some(__child_module))
            })
        }
    } else {
        quote! {
            {
                let __child_module = match __module.as_deref() {
                    ::core::option::Option::Some(__parent) => {
                        ::std::format!("{}.{}", __parent, #segment)
                    }
                    ::core::option::Option::None => ::std::string::String::from(#segment),
                };
                <#elem>::create_with_module(::core::option::Option::Some(__child_module))
            }
        }
    }
}

/// The body of `all_params`, walking nested structs and arrays.
fn expand_all_params(fields: &[Field]) -> TokenStream2 {
    let statements = fields.iter().map(|field| match field {
        Field::Param(leaf) => {
            let ident = &leaf.ident;
            quote! {
                __params.push(::rift_plugin::prelude::Param::as_ptr(&self.#ident));
            }
        }
        // Persist and skipped fields are host-invisible and stay out of `all_params`.
        Field::Persist(_) | Field::Skip(_) => quote! {},
        Field::Nested(nested) => {
            let ident = &nested.ident;
            if nested.array_len.is_some() {
                quote! {
                    for __item in self.#ident.iter() {
                        __params.extend(
                            ::rift_plugin::prelude::UserParams::all_params(__item),
                        );
                    }
                }
            } else {
                quote! {
                    __params.extend(
                        ::rift_plugin::prelude::UserParams::all_params(&self.#ident),
                    );
                }
            }
        }
    });

    quote! { #( #statements )* }
}

/// The body of `all_persist`, walking nested structs and arrays.
fn expand_all_persist(fields: &[Field]) -> TokenStream2 {
    let statements = fields.iter().map(|field| match field {
        Field::Persist(leaf) => {
            let ident = &leaf.ident;
            quote! {
                __persist.push(&self.#ident);
            }
        }
        Field::Nested(nested) => {
            let ident = &nested.ident;
            if nested.array_len.is_some() {
                quote! {
                    for __item in self.#ident.iter() {
                        __persist.extend(
                            ::rift_plugin::prelude::UserParams::all_persist(__item),
                        );
                    }
                }
            } else {
                quote! {
                    __persist.extend(
                        ::rift_plugin::prelude::UserParams::all_persist(&self.#ident),
                    );
                }
            }
        }
        Field::Param(_) | Field::Skip(_) => quote! {},
    });

    quote! { #( #statements )* }
}

/// Coerce an integer literal to a float literal for float parameters, so
/// `default = 1` and `range = linear(-60, 6)` work without an explicit `.0`.
fn coerce_float(expr: &Expr, kind: ParamKind) -> TokenStream2 {
    if kind != ParamKind::Float {
        return quote! { #expr };
    }

    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(int), ..
        }) => {
            let lit = LitFloat::new(&format!("{}.0", int.base10_digits()), int.span());
            quote! { #lit }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Float(float),
            ..
        }) => quote! { #float },
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr,
            ..
        }) => {
            let inner = coerce_float(expr, kind);
            quote! { - (#inner) }
        }
        other => quote! { (#other) as f32 },
    }
}

/// Render a type path as an expression path, adding turbofish `::` on generic
/// arguments so it can be used in value position (`Foo::<Bar>::builder()`).
fn type_path_expr(ty: &Type) -> TokenStream2 {
    let Type::Path(type_path) = ty else {
        return quote! { #ty };
    };

    let mut path = type_path.path.clone();
    for segment in path.segments.iter_mut() {
        if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = Some(Default::default());
        }
    }
    quote! { #path }
}
