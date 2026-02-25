use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, parse_macro_input};

#[derive(FromField)]
#[darling(attributes(builder), forward_attrs(doc))]
struct FieldReceiver {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,

    /// If not present, it defaults to None.
    #[darling(default)]
    default: Option<syn::Expr>,

    #[darling(default)]
    with: Option<syn::Expr>,
}

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

    let parsed_fields = fields
        .iter()
        .map(|field| FieldReceiver::from_field(field).unwrap())
        .collect::<Vec<_>>();

    // Every field except lens and accessor gets a setter
    let builder_fields = parsed_fields
        .iter()
        .filter(|f| {
            let name = f.ident.as_ref().unwrap().to_string();
            name != "lens" && name != "accessor"
        })
        .collect::<Vec<_>>();

    let builder_inits = builder_fields.iter().map(|f| {
        let fname = &f.ident;
        let default_value = f
            .default
            .clone()
            .map(|d| quote! { #d })
            .unwrap_or(quote! { Default::default() });

        quote! { #fname: #default_value }
    });

    let setters = builder_fields.iter().map(|f| {
        let fname = &f.ident;
        let fty = &f.ty;
        let docs = &f.attrs;

        // checkout if it's in option
        let (setter_ty, is_option) = match inner_type_of_option(fty) {
            Some(inner) => (inner, true),
            None => (fty, false),
        };

        // transform with "with"
        let transformed_value = if let Some(with_closure) = &f.with {
            quote! { (#with_closure)(value) }
        } else {
            quote! { value }
        };

        // if it's an option, we must wrap into Some
        let assignment = if is_option {
            quote! { Some(#transformed_value) }
        } else {
            quote! { #transformed_value }
        };

        quote! {
            #(#docs)*
            pub fn #fname(mut self, value: #setter_ty) -> Self {
                self.#fname = #assignment;
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
