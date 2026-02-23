use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Meta, parse_macro_input};

#[proc_macro_derive(ParamBuilder, attributes(builder))]
pub fn derive_param_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();

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
        quote! {
            pub fn #fname(mut self, value: #fty) -> Self {
                self.#fname = value;
                self
            }
        }
    });

    quote! {
        impl<L, MapFn, P> #name #ty_generics
        where
            P: Clone,
            L: Lens<Target = P> + Copy,
            MapFn: (Fn(&P) -> &dyn ClapParam) + Copy + 'static,
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

        impl<L, MapFn> View for #name <L, MapFn>
        where
            L: 'static,
            MapFn: 'static,
        {
        }
    }
    .into()
}
