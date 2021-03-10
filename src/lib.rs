use darling::FromMeta;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use std::iter::{self, FromIterator};
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Ident, ItemStruct};

/// Top-level configuration via the `superstruct` attribute.
#[derive(Debug, FromMeta)]
struct StructOpts {
    /// List of variant names of the superstruct being derived.
    variants: HashMap<Ident, ()>,
}

/// Field-level configuration.
#[derive(Debug, FromMeta)]
struct FieldOpts {
    only: HashMap<Ident, ()>,
}

#[proc_macro_attribute]
pub fn superstruct(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as ItemStruct);

    let name = &item.ident;
    // FIXME: use generics
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    let opts = StructOpts::from_list(&attr_args).unwrap();

    let mut output_items: Vec<TokenStream> = vec![];

    let mk_struct_name = |variant_name: &Ident| {
        Ident::new(
            &format!("{}{}", name, variant_name.to_string()),
            variant_name.span(),
        )
    };

    let variant_names = opts.variants.keys().cloned().collect_vec();
    let struct_names = variant_names.iter().map(mk_struct_name).collect_vec();

    // Map from variant to variant fields.
    let mut variant_fields =
        HashMap::<_, _>::from_iter(variant_names.iter().zip(iter::repeat(vec![])));

    for field in item.fields.iter() {
        // FIXME: filter out non superstruct attrs
        let field_variants = if let Some(field_opts) = field.attrs.iter().find_map(|attr| {
            let meta = attr.parse_meta().unwrap();
            Some(FieldOpts::from_meta(&meta).unwrap())
        }) {
            // Subset of fields.
            println!("{:#?}", field_opts);
            field_opts.only.keys().cloned().collect_vec()
        } else {
            // Common field -- all.
            variant_names.clone()
        };

        // XXX: nuke the attributes to prevent the compiler trying to expand our dodgy
        // configuration attributes
        // FIXME: shouldn't nuke all attributes
        let mut output_field = field.clone();
        output_field.attrs = vec![];

        for variant_name in field_variants {
            variant_fields
                .get_mut(&variant_name)
                .expect("invalid variant name in `only`")
                .push(output_field.clone());
        }
    }

    for (variant_name, struct_name) in variant_names.iter().zip(struct_names.iter()) {
        let fields = &variant_fields[variant_name];
        let variant_code = quote! {
            struct #struct_name {
                #(
                    #fields,
                )*
            }
        };
        output_items.push(variant_code.into());
    }

    let enum_item = quote! {
        enum #name {
            #(
                #variant_names(#struct_names),
            )*
        }
    };
    output_items.push(enum_item.into());

    TokenStream::from_iter(output_items)
}
