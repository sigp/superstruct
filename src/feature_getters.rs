use crate::{FeatureTypeOpts, VariantTypeOpts};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::Ident;

const DEFAULT_VARIANT_TYPE_GETTER: &str = "variant_type";
const DEFAULT_FEATURE_TYPE_LIST: &str = "list_all_features";
const DEFAULT_FEATURE_TYPE_CHECK: &str = "is_feature_enabled";

pub fn get_feature_getters(
    type_name: &Ident,
    variant_names: &[Ident],
    all_variant_features_opts: Option<HashMap<Ident, Vec<Ident>>>,
    variant_type_opts: &Option<VariantTypeOpts>,
    feature_type_opts: &Option<FeatureTypeOpts>,
) -> Vec<TokenStream> {
    let Some(variant_type) = variant_type_opts else {
        return vec![];
    };
    let Some(feature_type) = feature_type_opts else {
        return vec![];
    };
    let Some(all_variant_features) = all_variant_features_opts else {
        return vec![];
    };

    let mut output = vec![];

    output.extend(get_variant_type_getters(
        type_name,
        variant_names,
        variant_type,
    ));
    output.extend(get_feature_type_getters(
        type_name,
        all_variant_features,
        feature_type,
    ));
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

pub fn get_feature_type_getters(
    type_name: &Ident,
    all_variant_features: HashMap<Ident, Vec<Ident>>,
    feature_type: &FeatureTypeOpts,
) -> Vec<TokenStream> {
    let feature_type_name = &feature_type.name;
    let list_features = feature_type
        .list
        .clone()
        .unwrap_or_else(|| Ident::new(DEFAULT_FEATURE_TYPE_LIST, Span::call_site()));
    let all_variant_names: Vec<_> = all_variant_features.keys().collect();

    let mut feature_sets: Vec<Vec<Ident>> = vec![];

    for variant in all_variant_names.clone() {
        let feature_set: &Vec<Ident> = all_variant_features.get(variant).unwrap(); // TODO: Remove unwrap
        feature_sets.push(feature_set.clone());
    }

    let feature_list = quote! {
        fn #list_features(&self) -> &'static [#feature_type_name] {
            match self {
                #(
                    #type_name::#all_variant_names(..) => &[#(#feature_type_name::#feature_sets),*],
                )*
            }
        }
    };

    let check_feature = feature_type
        .check
        .clone()
        .unwrap_or_else(|| Ident::new(DEFAULT_FEATURE_TYPE_CHECK, Span::call_site()));
    let feature_check = quote! {
        fn #check_feature(&self, feature: #feature_type_name) -> bool {
            match self {
                #(
                    #type_name::#all_variant_names(..) => self.#list_features().contains(&feature),
                )*
            }
        }
    };
    vec![feature_list.into(), feature_check.into()]
}
