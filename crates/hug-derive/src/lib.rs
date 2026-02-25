use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Meta, parse_macro_input};

#[proc_macro_derive(ParamBuilder, attributes(builder))]
pub fn derive_param_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("named fields only"),
        },
        _ => panic!("structs only"),
    };

    // Every field except lens and accessor gets a setter
    let builder_fields: Vec<_> = fields
        .iter()
        .filter(|f| {
            let name = f.ident.as_ref().unwrap().to_string();
            name != "lens" && name != "accessor"
        })
        .collect();

    let builder_inits = builder_fields.iter().map(|f| {
        let fname = &f.ident;
        let default_val: proc_macro2::TokenStream = f
            .attrs
            .iter()
            .find(|a| a.path().is_ident("builder"))
            .and_then(|a| {
                if let Meta::List(list) = &a.meta {
                    let s = list.tokens.to_string();
                    let val = s.trim_start_matches("default =").trim().to_string();
                    Some(val.parse().unwrap())
                } else {
                    None
                }
            })
            .unwrap_or(quote! { Default::default() });
        quote! { #fname: #default_val }
    });

    let setters = builder_fields.iter().map(|f| {
        let fname = &f.ident;
        let fty = &f.ty;

        // If the field is Option<T>, the setter accepts T; otherwise it accepts fty
        let (setter_ty, assignment) = match inner_type_of_option(fty) {
            Some(inner_ty) => (inner_ty, quote! { self.#fname = Some(value); }),
            None => (fty, quote! { self.#fname = value; }),
        };

        quote! {
            pub fn #fname(mut self, value: #setter_ty) -> Self {
                #assignment
                self
            }
        }
    });

    let extra_where = if let Some(wc) = where_clause {
        let predicates = &wc.predicates;
        quote! { #predicates }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics #name #ty_generics
        where
            L: Lens + Copy,
            L::Target: Clone,
            MapFn: (Fn(&L::Target) -> &dyn ClapParam) + Copy + 'static,
            #extra_where
        {
            pub fn new(lens: L, accessor: MapFn) -> Self {
                Self {
                    lens,
                    accessor,
                    #(#builder_inits,)*
                }
            }

            pub fn lens_and_accessor(&self) -> (L, MapFn) {
                (self.lens, self.accessor)
            }
            #(#setters)*
        }

        impl #impl_generics View for #name #impl_generics #where_clause
        {
        }
    }
    .into()
}

fn inner_type_of_option(ty: &syn::Type) -> Option<&syn::Type> {
    // Must be a path type (not a reference, slice, etc.)
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    // Get the last segment, e.g. `Option` in `std::option::Option`
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Option" {
        return None;
    }
    // Must have angle bracket args: Option<T>
    let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else {
        return None;
    };
    // Grab the first generic arg and make sure it's a type
    let syn::GenericArgument::Type(inner_ty) = args.args.first()? else {
        return None;
    };

    Some(inner_ty)
}
