use crate::{FeatureTypeOpts, VariantTypeOpts};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

const DEFAULT_VARIANT_TYPE_GETTER: &str = "variant_type";

pub fn get_feature_getters(
    type_name: &Ident,
    variant_names: &[Ident],
    variant_type_opts: &Option<VariantTypeOpts>,
    feature_type_opts: &Option<FeatureTypeOpts>,
) -> Vec<TokenStream> {
    let Some(variant_type) = variant_type_opts else {
        return vec![];
    };
    let Some(feature_type) = feature_type_opts else {
        return vec![];
    };
    let mut output = vec![];

    output.extend(get_variant_type_getters(
        type_name,
        variant_names,
        variant_type,
    ));
    // output.extend(get_feature_type_getters(variant_type));
    output
}

pub fn get_variant_type_getters(
    type_name: &Ident,
    variant_names: &[Ident],
    variant_type: &VariantTypeOpts,
) -> Vec<TokenStream> {
    let variant_type_name = &variant_type.name;
    let getter_name = variant_type
        .getter
        .clone()
        .unwrap_or_else(|| Ident::new(DEFAULT_VARIANT_TYPE_GETTER, Span::call_site()));
    let getter = quote! {
        fn #getter_name(&self) -> #variant_type_name {
            match self {
                #(
                    #type_name::#variant_names(..) => #variant_type_name::#variant_names,
                )*
            }
        }
    };
    vec![getter.into()]
}
