use std::{
    collections::HashMap,
    iter::{self, FromIterator},
};

use attributes::{IdentList, NestedMetaList};
use darling::{export::NestedMeta, util::Override, FromMeta};
use from::{
    generate_from_enum_trait_impl_for_ref, generate_from_variant_trait_impl,
    generate_from_variant_trait_impl_for_ref,
};
use itertools::Itertools;
use macros::generate_all_map_macros;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, Attribute, Expr, Field, GenericParam, Ident, ItemStruct, Lifetime,
    LifetimeParam, Type, TypeGenerics, TypeParamBound,
};

mod attributes;
mod from;
mod macros;
mod naming;
mod utils;

/// Top-level configuration via the `superstruct` attribute.
#[derive(Debug, FromMeta)]
struct StructOpts {
    /// List of meta variant names of the superstruct being derived.
    #[darling(default)]
    meta_variants: Option<IdentList>,
    /// List of variant names of the superstruct being derived.
    variants: IdentList,
    /// List of attributes to apply to the variant structs.
    #[darling(default)]
    variant_attributes: Option<NestedMetaList>,
    /// List of attributes to apply to a selection of named variant structs.
    #[darling(default)]
    specific_variant_attributes: Option<HashMap<Ident, NestedMetaList>>,
    /// List of attributes to apply to the generated Ref type.
    #[darling(default)]
    ref_attributes: Option<NestedMetaList>,
    /// List of attributes to apply to the generated RefMut type.
    #[darling(default)]
    ref_mut_attributes: Option<NestedMetaList>,
    /// Error type and expression to use for casting methods.
    #[darling(default)]
    cast_error: ErrorOpts,
    /// Error type and expression to use for partial getter methods.
    #[darling(default)]
    partial_getter_error: ErrorOpts,
    /// Turn off the generation of the top-level enum that binds the variants together.
    #[darling(default)]
    no_enum: bool,
    /// Turn off the generation of the map macros.
    #[darling(default)]
    no_map_macros: bool,
    /// List of other superstruct types to generate (owned) mappings into.
    #[darling(default)]
    map_into: Option<IdentList>,
    /// List of other superstruct types to generate mappings into from Ref.
    #[darling(default)]
    map_ref_into: Option<IdentList>,
    /// List of other superstruct types to generate mappings into from RefMut.
    #[darling(default)]
    map_ref_mut_into: Option<IdentList>,
}

/// Field-level configuration.
#[derive(Debug, Default, FromMeta)]
struct FieldOpts {
    // TODO: When we update darling, we can replace `Override`
    // with a custom enum and use `#[darling(word)]` on the variant
    // we want to use for `#[superstruct(flatten)]` (no variants specified).
    #[darling(default)]
    flatten: Option<Override<HashMap<Ident, ()>>>,
    #[darling(default)]
    only: Option<HashMap<Ident, ()>>,
    #[darling(default)]
    meta_only: Option<HashMap<Ident, ()>>,
    #[darling(default)]
    getter: Option<GetterOpts>,
    #[darling(default)]
    partial_getter: Option<GetterOpts>,
}

/// Getter configuration for a specific field
#[derive(Debug, Default, FromMeta)]
struct GetterOpts {
    #[darling(default)]
    copy: bool,
    #[darling(default)]
    no_mut: bool,
    #[darling(default)]
    rename: Option<Ident>,
}

#[derive(Debug, Default, FromMeta)]
struct ErrorOpts {
    #[darling(default)]
    ty: Option<String>,
    #[darling(default)]
    expr: Option<String>,
}

impl ErrorOpts {
    fn parse(&self) -> Option<(Type, Expr)> {
        let err_ty_str = self.ty.as_ref()?;
        let err_ty: Type = syn::parse_str(err_ty_str).expect("error type not valid");
        let err_expr_str = self
            .expr
            .as_ref()
            .expect("must provide an error expr with error ty");
        let err_expr: Expr = syn::parse_str(err_expr_str).expect("error expr not valid");
        Some((err_ty, err_expr))
    }

    fn build_result_type(
        &self,
        ret_ty: impl ToTokens,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        if let Some((err_ty, err_expr)) = self.parse() {
            (quote! { Result<#ret_ty, #err_ty> }, quote! { #err_expr })
        } else {
            (quote! { Result<#ret_ty, ()> }, quote! { () })
        }
    }
}

/// All data about a field, including its type & config from attributes.
#[derive(Debug)]
struct FieldData {
    name: Ident,
    field: Field,
    only_combinations: Vec<VariantKey>,
    getter_opts: GetterOpts,
    partial_getter_opts: GetterOpts,
    is_common: bool,
}

impl FieldData {
    fn is_common(&self) -> bool {
        self.is_common
    }

    /// Checks whether this field should be included in creating
    /// partial getters for the given type name.
    fn exists_in_meta(&self, type_name: &Ident) -> bool {
        let only_metas = self
            .only_combinations
            .iter()
            .filter_map(|only| only.meta_variant.as_ref())
            .map(ToString::to_string)
            .collect::<Vec<_>>();

        if only_metas.is_empty() {
            return true;
        }
        only_metas
            .iter()
            .any(|only| type_name.to_string().ends_with(only))
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct VariantKey {
    variant: Ident,
    meta_variant: Option<Ident>,
}

#[proc_macro_attribute]
pub fn superstruct(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error().into(),
    };
    let item = parse_macro_input!(input as ItemStruct);

    let type_name = &item.ident;
    let visibility = item.vis.clone();
    // Extract the generics to use for the top-level type and all variant structs.
    let decl_generics = &item.generics;
    // Generics used for the impl block.
    let (_, _, where_clause) = &item.generics.split_for_impl();

    let opts = StructOpts::from_list(&attr_args).unwrap();

    let mut output_items: Vec<TokenStream> = vec![];

    let mk_struct_name = |variant_key: &VariantKey| {
        let VariantKey {
            variant,
            meta_variant,
        } = variant_key;

        if let Some(meta_variant) = meta_variant {
            format_ident!("{}{}{}", type_name, meta_variant, variant)
        } else {
            format_ident!("{}{}", type_name, variant)
        }
    };

    let variant_names = &opts.variants.idents;
    let meta_variant_names = &opts
        .meta_variants
        .clone()
        .map(|mv| mv.idents.into_iter().map(Some).collect_vec())
        .unwrap_or(vec![None]);
    let variant_combinations = variant_names
        .iter()
        .cloned()
        .cartesian_product(meta_variant_names.iter().cloned())
        .map(|(v, mv)| VariantKey {
            variant: v,
            meta_variant: mv,
        });

    let struct_names = variant_combinations
        .clone()
        .map(|key| mk_struct_name(&key))
        .collect_vec();

    // Vec of field data.
    let mut fields = vec![];
    // Map from variant or meta variant to variant fields.
    let mut variant_fields =
        HashMap::<_, _>::from_iter(variant_combinations.clone().zip(iter::repeat(vec![])));

    for field in &item.fields {
        let name = field.ident.clone().expect("named fields only");
        let field_opts = field
            .attrs
            .iter()
            .filter(|attr| is_superstruct_attr(attr))
            .map(|attr| FieldOpts::from_meta(&attr.meta).unwrap())
            .next()
            .unwrap_or_default();

        // Check for conflicting attributes.
        check_for_conflicting_superstruct_attrs(&field.attrs);

        // Drop the field-level superstruct attributes
        let mut output_field = field.clone();
        output_field.attrs = discard_superstruct_attrs(&output_field.attrs);

        // Add the field to the `variant_fields` map for all applicable variants.
        let field_variants = field_opts.only.as_ref().map_or_else(
            || variant_names.clone(),
            |only| only.keys().cloned().collect_vec(),
        );
        let field_meta_variants = field_opts.meta_only.as_ref().map_or_else(
            || meta_variant_names.clone(),
            |meta_only| meta_only.keys().cloned().map(Some).collect_vec(),
        );

        // Field is common if it is part of every meta variant AND every variant.
        let is_common_meta = opts
            .meta_variants
            .as_ref()
            .map_or(true, |struct_meta_variants| {
                struct_meta_variants.idents.len() == field_meta_variants.len()
            });
        let is_common = field_variants.len() == variant_names.len() && is_common_meta;

        let only_combinations = field_variants
            .iter()
            .cartesian_product(field_meta_variants.iter());

        for (variant, meta_variant) in only_combinations.clone() {
            variant_fields
                .get_mut(&VariantKey {
                    variant: variant.clone(),
                    meta_variant: meta_variant.clone(),
                })
                .expect("invalid variant name in `only` or `meta_only`")
                .push(output_field.clone());
        }

        // Check field opts
        if field_opts.only.is_some() && field_opts.getter.is_some() {
            panic!("can't configure `only` and `getter` on the same field");
        } else if field_opts.meta_only.is_some() && field_opts.getter.is_some() {
            panic!("can't configure `meta_only` and `getter` on the same field");
        } else if field_opts.only.is_none()
            && field_opts.meta_only.is_none()
            && field_opts.partial_getter.is_some()
        {
            panic!("can't set `partial_getter` options on common field");
        } else if field_opts.flatten.is_some() && field_opts.only.is_some() {
            panic!("can't set `flatten` and `only` on the same field");
        } else if field_opts.flatten.is_some() && field_opts.getter.is_some() {
            panic!("can't set `flatten` and `getter` on the same field");
        } else if field_opts.flatten.is_some() && field_opts.partial_getter.is_some() {
            panic!("can't set `flatten` and `partial_getter` on the same field");
        }

        let getter_opts = field_opts.getter.unwrap_or_default();
        let partial_getter_opts = field_opts.partial_getter.unwrap_or_default();

        if let Some(flatten_opts) = field_opts.flatten {
            for variant_key in variant_combinations.clone() {
                let variant = &variant_key.variant;
                let meta_variant = variant_key.meta_variant.as_ref();

                let Some(variant_field_index) = variant_fields
                    .get(&variant_key)
                    .expect("invalid variant name")
                    .iter()
                    .position(|f| f.ident.as_ref() == Some(&name))
                else {
                    continue;
                };

                if should_skip(
                    variant_names,
                    meta_variant_names,
                    &flatten_opts,
                    &variant_key,
                ) {
                    // Remove the field from the field map
                    let fields = variant_fields
                        .get_mut(&variant_key)
                        .expect("invalid variant name");
                    fields.remove(variant_field_index);
                    continue;
                }

                // Update the struct name for this variant.
                let mut next_variant_field = output_field.clone();

                let last_segment_mut_ref = match next_variant_field.ty {
                    Type::Path(ref mut p) => {
                        &mut p
                            .path
                            .segments
                            .last_mut()
                            .expect("path should have at least one segment")
                            .ident
                    }
                    _ => panic!("field must be a path"),
                };

                let (next_variant_ty_name, partial_getter_rename) =
                    if let Some(meta_variant) = meta_variant {
                        if let Some(meta_only) = field_opts.meta_only.as_ref() {
                            assert_eq!(
                                meta_only.len(),
                                1,
                                "when used in combination with flatten, only \
                                one meta variant specification is allowed"
                            );
                            assert_eq!(
                                meta_only.keys().next().unwrap(),
                                meta_variant,
                                "flattened meta variant does not match"
                            );
                            (
                                format_ident!("{}{}", last_segment_mut_ref.clone(), variant),
                                format_ident!("{}_{}", name, variant.to_string().to_lowercase()),
                            )
                        } else {
                            (
                                format_ident!(
                                    "{}{}{}",
                                    last_segment_mut_ref.clone(),
                                    meta_variant,
                                    variant
                                ),
                                format_ident!(
                                    "{}_{}_{}",
                                    name,
                                    meta_variant.to_string().to_lowercase(),
                                    variant.to_string().to_lowercase()
                                ),
                            )
                        }
                    } else {
                        (
                            format_ident!("{}{}", last_segment_mut_ref.clone(), variant),
                            format_ident!("{}_{}", name, variant.to_string().to_lowercase()),
                        )
                    };
                *last_segment_mut_ref = next_variant_ty_name;

                // Create a partial getter for the field.
                let partial_getter_opts = GetterOpts {
                    rename: Some(partial_getter_rename),
                    ..<_>::default()
                };

                fields.push(FieldData {
                    name: name.clone(),
                    field: next_variant_field.clone(),
                    // Make sure the field is only accessible from this variant.
                    only_combinations: vec![variant_key.clone()],
                    getter_opts: <_>::default(),
                    partial_getter_opts,
                    is_common: false,
                });

                // Update the variant field map
                let fields = variant_fields
                    .get_mut(&variant_key)
                    .expect("invalid variant name");
                *fields
                    .get_mut(variant_field_index)
                    .expect("invalid field index") = next_variant_field;
            }
        } else {
            fields.push(FieldData {
                name,
                field: output_field,
                only_combinations: only_combinations
                    .map(|(variant, meta_variant)| VariantKey {
                        variant: variant.clone(),
                        meta_variant: meta_variant.clone(),
                    })
                    .collect_vec(),
                getter_opts,
                partial_getter_opts,
                is_common,
            });
        }
    }

    // Generate structs for all of the variants.
    let universal_struct_attributes = opts
        .variant_attributes
        .as_ref()
        .map_or(&[][..], |attrs| &attrs.metas);

    for (variant_key, struct_name) in variant_combinations.zip(struct_names.iter()) {
        let fields = &variant_fields[&variant_key];

        let specific_struct_attributes = opts
            .specific_variant_attributes
            .as_ref()
            .and_then(|sv| sv.get(&variant_key.variant))
            .map_or(&[][..], |attrs| &attrs.metas);
        let specific_struct_attributes_meta = opts
            .specific_variant_attributes
            .as_ref()
            .and_then(|sv| variant_key.meta_variant.and_then(|mv| sv.get(&mv)))
            .map_or(&[][..], |attrs| &attrs.metas);
        let spatt = specific_struct_attributes
            .iter()
            .chain(specific_struct_attributes_meta.iter());

        let variant_code = quote! {
            #(
                #[#universal_struct_attributes]
            )*
            #(
                #[#spatt]
            )*
            #visibility struct #struct_name #decl_generics #where_clause {
                #(
                    #fields,
                )*
            }
        };
        output_items.push(variant_code.into());
    }

    // If the `no_enum` attribute is set, stop after generating variant structs.
    if opts.no_enum {
        return TokenStream::from_iter(output_items);
    }

    let mut inner_enum_names = vec![];

    // Generate inner enums if necessary.
    for meta_variant in meta_variant_names.iter().flatten() {
        let inner_enum_name = format_ident!("{}{}", type_name, meta_variant);
        inner_enum_names.push(inner_enum_name.clone());
        let inner_struct_names = variant_names
            .iter()
            .map(|variant_name| format_ident!("{}{}", inner_enum_name, variant_name))
            .collect_vec();
        generate_wrapper_enums(
            &inner_enum_name,
            &item,
            &opts,
            &mut output_items,
            variant_names,
            &inner_struct_names,
            &fields,
            false,
        );
    }

    // Generate outer enum.
    let variant_names = opts
        .meta_variants
        .as_ref()
        .map(|mv| &mv.idents)
        .unwrap_or(variant_names);
    let struct_names = &opts
        .meta_variants
        .as_ref()
        .map(|_| inner_enum_names)
        .unwrap_or(struct_names);
    generate_wrapper_enums(
        type_name,
        &item,
        &opts,
        &mut output_items,
        variant_names,
        struct_names,
        &fields,
        opts.meta_variants.is_some(),
    );

    TokenStream::from_iter(output_items)
}

#[allow(clippy::too_many_arguments)]
fn generate_wrapper_enums(
    type_name: &Ident,
    item: &ItemStruct,
    opts: &StructOpts,
    output_items: &mut Vec<TokenStream>,
    variant_names: &[Ident],
    struct_names: &[Ident],
    fields: &[FieldData],
    is_meta: bool,
) {
    let visibility = &item.vis;
    // Extract the generics to use for the top-level type and all variant structs.
    let decl_generics = &item.generics;
    // Generics used for the impl block.
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    // Construct the top-level enum.
    let top_level_attrs = discard_superstruct_attrs(&item.attrs);
    let enum_item = quote! {
        #(
            #top_level_attrs
        )*
        #visibility enum #type_name #decl_generics #where_clause {
            #(
                #variant_names(#struct_names #ty_generics),
            )*
        }
    };
    output_items.push(enum_item.into());

    // Construct a top-level reference type.
    // TODO: check that variants aren't called `Ref`
    let ref_ty_name = format_ident!("{}Ref", type_name);
    let ref_ty_lifetime = Lifetime::new("'__superstruct", Span::call_site());

    // Muahaha, this is dank.
    // Inject the generated lifetime into the top-level type's generics.
    let mut ref_ty_decl_generics = decl_generics.clone();
    ref_ty_decl_generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(ref_ty_lifetime.clone())),
    );

    // If no lifetime bound exists for a generic param, inject one.
    for param in ref_ty_decl_generics.params.iter_mut() {
        if let GenericParam::Type(type_param) = param {
            let result = type_param
                .bounds
                .iter()
                .find(|bound| matches!(bound, TypeParamBound::Lifetime(_)));
            if result.is_none() {
                type_param
                    .bounds
                    .insert(0, TypeParamBound::Lifetime(ref_ty_lifetime.clone()))
            }
        }
    }

    let (ref_impl_generics, ref_ty_generics, _) = &ref_ty_decl_generics.split_for_impl();

    // Prepare the attributes for the ref type.
    let ref_attributes = opts
        .ref_attributes
        .as_ref()
        .map_or(&[][..], |attrs| &attrs.metas);

    let ref_ty = quote! {
        #(
            #[#ref_attributes]
        )*
        #visibility enum #ref_ty_name #ref_ty_decl_generics #where_clause {
            #(
                #variant_names(&#ref_ty_lifetime #struct_names #ty_generics),
            )*
        }
    };
    output_items.push(ref_ty.into());

    // Construct a top-level mutable reference type.
    // TODO: check that variants aren't called `RefMut`
    let ref_mut_ty_name = format_ident!("{}RefMut", type_name);
    let ref_mut_ty_lifetime = Lifetime::new("'__superstruct", Span::call_site());
    // Muahaha, this is dank.
    // Inject the generated lifetime into the top-level type's generics.
    let mut ref_mut_ty_decl_generics = decl_generics.clone();
    ref_mut_ty_decl_generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam::new(ref_mut_ty_lifetime.clone())),
    );
    let (ref_mut_impl_generics, ref_mut_ty_generics, _) =
        &ref_mut_ty_decl_generics.split_for_impl();

    // Prepare the attributes for the ref type.
    let ref_mut_attributes = opts
        .ref_mut_attributes
        .as_ref()
        .map_or(&[][..], |attrs| &attrs.metas);

    let ref_mut_ty = quote! {
        #(
            #[#ref_mut_attributes]
        )*
        #visibility enum #ref_mut_ty_name #ref_mut_ty_decl_generics #where_clause {
            #(
                #variant_names(&#ref_mut_ty_lifetime mut #struct_names #ty_generics),
            )*
        }
    };
    output_items.push(ref_mut_ty.into());

    // Construct the main impl block.
    let getters = fields
        .iter()
        .filter(|f| f.is_common())
        .map(|field_data| make_field_getter(type_name, variant_names, field_data, None, is_meta));

    let mut_getters = fields
        .iter()
        .filter(|f| f.is_common() && !f.getter_opts.no_mut)
        .map(|field_data| {
            make_mut_field_getter(type_name, variant_names, field_data, None, is_meta)
        });

    let partial_getters = fields
        .iter()
        .filter(|f| !f.is_common())
        .filter(|f| is_meta || f.exists_in_meta(type_name))
        .cartesian_product(&[false, true])
        .flat_map(|(field_data, mutability)| {
            let field_variants = &field_data.only_combinations;
            Some(make_partial_getter(
                type_name,
                field_data,
                field_variants,
                &opts.partial_getter_error,
                *mutability,
                None,
                is_meta,
            ))
        });

    let cast_methods = variant_names
        .iter()
        .flat_map(|variant_name| {
            let caster = make_as_variant_method(
                type_name,
                variant_name,
                ty_generics,
                &opts.cast_error,
                false,
            );
            let caster_mut = make_as_variant_method(
                type_name,
                variant_name,
                ty_generics,
                &opts.cast_error,
                true,
            );
            vec![caster, caster_mut]
        })
        .collect_vec();

    let impl_block = quote! {
        impl #impl_generics #type_name #ty_generics #where_clause {
            pub fn to_ref<#ref_ty_lifetime>(&#ref_ty_lifetime self) -> #ref_ty_name #ref_ty_generics {
                match self {
                    #(
                        #type_name::#variant_names(ref inner)
                            => #ref_ty_name::#variant_names(inner),
                    )*
                }
            }
            pub fn to_mut<#ref_mut_ty_lifetime>(&#ref_mut_ty_lifetime mut self) -> #ref_mut_ty_name #ref_mut_ty_generics {
                match self {
                    #(
                        #type_name::#variant_names(ref mut inner)
                            => #ref_mut_ty_name::#variant_names(inner),
                    )*
                }
            }
            #(
                #cast_methods
            )*
            #(
                #getters
            )*
            #(
                #mut_getters
            )*
            #(
                #partial_getters
            )*
        }
    };
    output_items.push(impl_block.into());

    // Construct the impl block for the *Ref type.
    let ref_getters = fields.iter().filter(|f| f.is_common()).map(|field_data| {
        make_field_getter(
            &ref_ty_name,
            variant_names,
            field_data,
            Some(&ref_ty_lifetime),
            is_meta,
        )
    });

    let ref_partial_getters = fields
        .iter()
        .filter(|f| !f.is_common())
        .filter(|f| is_meta || f.exists_in_meta(type_name))
        .flat_map(|field_data| {
            let field_variants = &field_data.only_combinations;
            Some(make_partial_getter(
                &ref_ty_name,
                field_data,
                field_variants,
                &opts.partial_getter_error,
                false,
                Some(&ref_ty_lifetime),
                is_meta,
            ))
        });

    let ref_impl_block = quote! {
        impl #ref_impl_generics #ref_ty_name #ref_ty_generics #where_clause {
            #(
                #ref_getters
            )*

            #(
                #ref_partial_getters
            )*
        }

        // Reference types are just wrappers around references, so they can be copied!
        impl #ref_impl_generics Copy for #ref_ty_name #ref_ty_generics #where_clause { }
        impl #ref_impl_generics Clone for #ref_ty_name #ref_ty_generics #where_clause {
            fn clone(&self) -> Self { *self }
        }
    };
    output_items.push(ref_impl_block.into());

    // Construct the impl block for the *RefMut type.
    let ref_mut_getters = fields
        .iter()
        .filter(|f| f.is_common() && !f.getter_opts.no_mut)
        .map(|field_data| {
            make_mut_field_getter(
                &ref_mut_ty_name,
                variant_names,
                field_data,
                Some(&ref_mut_ty_lifetime),
                is_meta,
            )
        });

    let ref_mut_partial_getters = fields
        .iter()
        .filter(|f| !f.is_common() && !f.partial_getter_opts.no_mut)
        .filter(|f| is_meta || f.exists_in_meta(type_name))
        .flat_map(|field_data| {
            let field_variants = &field_data.only_combinations;
            Some(make_partial_getter(
                &ref_mut_ty_name,
                field_data,
                field_variants,
                &opts.partial_getter_error,
                true,
                Some(&ref_mut_ty_lifetime),
                is_meta,
            ))
        });

    let ref_mut_impl_block = quote! {
        impl #ref_mut_impl_generics #ref_mut_ty_name #ref_mut_ty_generics #where_clause {
            #(
                #ref_mut_getters
            )*

            #(
                #ref_mut_partial_getters
            )*
        }
    };
    output_items.push(ref_mut_impl_block.into());

    // Generate the mapping macros if enabled.
    if !opts.no_map_macros && !opts.no_enum {
        let num_generics = decl_generics.params.len();
        generate_all_map_macros(
            type_name,
            &ref_ty_name,
            &ref_mut_ty_name,
            num_generics,
            struct_names,
            variant_names,
            opts,
            output_items,
        );
    } else {
        assert!(
            opts.map_into.is_none(),
            "`map_into` is set but map macros are disabled"
        );
        assert!(
            opts.map_ref_into.is_none(),
            "`map_ref_into` is set but map macros are disabled"
        );
        assert!(
            opts.map_ref_mut_into.is_none(),
            "`map_ref_mut_into` is set but map macros are disabled"
        );
    }

    // Generate trait implementations.
    for (variant_name, struct_name) in variant_names.iter().zip_eq(struct_names) {
        let from_impl = generate_from_variant_trait_impl(
            type_name,
            impl_generics,
            ty_generics,
            where_clause,
            variant_name,
            struct_name,
        );
        output_items.push(from_impl.into());

        let from_impl_for_ref = generate_from_variant_trait_impl_for_ref(
            &ref_ty_name,
            &ref_ty_lifetime,
            ref_impl_generics,
            ref_ty_generics,
            ty_generics,
            where_clause,
            variant_name,
            struct_name,
        );
        output_items.push(from_impl_for_ref.into());
    }

    // Convert reference to top-level type to `Ref`.
    let ref_from_top_level_impl = generate_from_enum_trait_impl_for_ref(
        type_name,
        ty_generics,
        &ref_ty_name,
        &ref_ty_lifetime,
        ref_impl_generics,
        ref_ty_generics,
        where_clause,
    );
    output_items.push(ref_from_top_level_impl.into());
}

/// Generate a getter method for a field.
fn make_field_getter(
    type_name: &Ident,
    variant_names: &[Ident],
    field_data: &FieldData,
    lifetime: Option<&Lifetime>,
    is_meta: bool,
) -> proc_macro2::TokenStream {
    let field_name = &field_data.name;
    let field_type = &field_data.field.ty;
    let getter_opts = &field_data.getter_opts;

    let fn_name = getter_opts.rename.as_ref().unwrap_or(field_name);
    let return_type = if getter_opts.copy {
        quote! { #field_type }
    } else {
        quote! { &#lifetime #field_type}
    };

    let return_expr = if is_meta {
        quote! { inner.#field_name() }
    } else if getter_opts.copy {
        quote! { inner.#field_name }
    } else {
        quote! { &inner.#field_name }
    };

    // Pass-through `cfg` attributes as they affect the existence of this field.
    let cfg_attrs = get_cfg_attrs(&field_data.field.attrs);

    quote! {
        #(
            #cfg_attrs
        )*
        pub fn #fn_name(&self) -> #return_type {
            match self {
                #(
                    #type_name::#variant_names(ref inner) => {
                        #return_expr
                    }
                )*
            }
        }
    }
}

/// Generate a mutable getter method for a field.
fn make_mut_field_getter(
    type_name: &Ident,
    variant_names: &[Ident],
    field_data: &FieldData,
    lifetime: Option<&Lifetime>,
    is_meta: bool,
) -> proc_macro2::TokenStream {
    let field_name = &field_data.name;
    let field_type = &field_data.field.ty;
    let getter_opts = &field_data.getter_opts;

    let fn_name = format_ident!("{}_mut", getter_opts.rename.as_ref().unwrap_or(field_name));
    let return_type = quote! { &#lifetime mut #field_type };
    let param = make_self_arg(true, lifetime);
    let return_expr = if is_meta {
        quote! { inner.#fn_name() }
    } else {
        quote! { &mut inner.#field_name }
    };

    // Pass-through `cfg` attributes as they affect the existence of this field.
    let cfg_attrs = get_cfg_attrs(&field_data.field.attrs);

    quote! {
        #(
            #cfg_attrs
        )*
        pub fn #fn_name(#param) -> #return_type {
            match self {
                #(
                    #type_name::#variant_names(ref mut inner) => {
                        #return_expr
                    }
                )*
            }
        }
    }
}

fn make_self_arg(mutable: bool, lifetime: Option<&Lifetime>) -> proc_macro2::TokenStream {
    if mutable {
        quote! { &#lifetime mut self }
    } else {
        // Ignore the lifetime for immutable references. This allows `&Ref<'a>` to be de-referenced
        // to an inner pointer with lifetime `'a`.
        quote! { &self }
    }
}

fn make_type_ref(
    ty: &Type,
    mutable: bool,
    copy: bool,
    lifetime: Option<&Lifetime>,
) -> proc_macro2::TokenStream {
    // XXX: bit hacky, ignore `copy` if `mutable` is set
    if mutable {
        quote! { &#lifetime mut #ty }
    } else if copy {
        quote! { #ty }
    } else {
        quote! { &#lifetime #ty }
    }
}

/// Generate a partial getter method for a field.
fn make_partial_getter(
    type_name: &Ident,
    field_data: &FieldData,
    field_variants: &[VariantKey],
    error_opts: &ErrorOpts,
    mutable: bool,
    lifetime: Option<&Lifetime>,
    is_meta: bool,
) -> proc_macro2::TokenStream {
    let field_variants = field_variants
        .iter()
        .filter_map(|key| {
            if is_meta {
                key.meta_variant.clone()
            } else {
                Some(key.variant.clone())
            }
        })
        .unique()
        .collect_vec();
    let type_name = type_name.clone();

    let field_name = &field_data.name;
    let renamed_field = field_data
        .partial_getter_opts
        .rename
        .as_ref()
        .unwrap_or(field_name);
    let fn_name = if mutable {
        format_ident!("{}_mut", renamed_field)
    } else {
        renamed_field.clone()
    };
    let copy = field_data.partial_getter_opts.copy;
    let self_arg = make_self_arg(mutable, lifetime);
    let ret_ty = make_type_ref(&field_data.field.ty, mutable, copy, lifetime);
    let ret_expr = if is_meta {
        quote! { inner.#fn_name()? }
    } else if mutable {
        quote! { &mut inner.#field_name }
    } else if copy {
        quote! { inner.#field_name }
    } else {
        quote! { &inner.#field_name }
    };
    let (res_ret_ty, err_expr) = error_opts.build_result_type(&ret_ty);

    // Pass-through `cfg` attributes as they affect the existence of this field.
    let cfg_attrs = get_cfg_attrs(&field_data.field.attrs);

    quote! {
        #(
            #cfg_attrs
        )*
        pub fn #fn_name(#self_arg) -> #res_ret_ty {
            match self {
                #(
                    #type_name::#field_variants(inner) => Ok(#ret_expr),
                )*
                _ => Err(#err_expr),
            }
        }
    }
}

/// Generate a `as_<variant_name>{_mut}` method.
fn make_as_variant_method(
    type_name: &Ident,
    variant_name: &Ident,
    type_generics: &TypeGenerics,
    cast_err_opts: &ErrorOpts,
    mutable: bool,
) -> proc_macro2::TokenStream {
    let variant_ty = format_ident!("{}{}", type_name, variant_name);
    let (suffix, arg, ret_ty, binding) = if mutable {
        (
            "_mut",
            quote! { &mut self },
            quote! { &mut #variant_ty #type_generics },
            quote! { ref mut inner },
        )
    } else {
        (
            "",
            quote! { &self },
            quote! { &#variant_ty #type_generics },
            quote! { ref inner },
        )
    };
    let (ret_res_ty, err_expr) = cast_err_opts.build_result_type(&ret_ty);
    let fn_name = format_ident!("as_{}{}", variant_name.to_string().to_lowercase(), suffix);
    quote! {
        pub fn #fn_name(#arg) -> #ret_res_ty {
            match self {
                #type_name::#variant_name(#binding) => Ok(inner),
                _ => Err(#err_expr),
            }
        }
    }
}

/// Check that there is at most one superstruct attribute, and panic otherwise.
fn check_for_conflicting_superstruct_attrs(attrs: &[Attribute]) {
    if attrs
        .iter()
        .filter(|attr| is_superstruct_attr(attr))
        .count()
        > 1
    {
        // TODO: this is specific to fields right now, but we could maybe make it work for the
        // top-level attributes. I'm just not sure how to get at them under the `AttributeArgs`
        // stuff.
        panic!("cannot handle more than one superstruct attribute per field");
    }
}

/// Keep all non-superstruct-related attributes from an array.
fn discard_superstruct_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !is_superstruct_attr(attr))
        .cloned()
        .collect()
}

/// Keep only `cfg` attributes from an array.
fn get_cfg_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| is_attr_with_ident(attr, "cfg"))
        .cloned()
        .collect()
}

/// Predicate for determining whether an attribute is a `superstruct` attribute.
fn is_superstruct_attr(attr: &Attribute) -> bool {
    is_attr_with_ident(attr, "superstruct")
}

/// Predicate for determining whether an attribute has the given `ident` as its path.
fn is_attr_with_ident(attr: &Attribute, ident: &str) -> bool {
    attr.path()
        .get_ident()
        .map_or(false, |attr_ident| *attr_ident == ident)
}

/// Predicate for determining whether a field should be excluded from a flattened
/// variant combination.
fn should_skip(
    variant_names: &[Ident],
    meta_variant_names: &[Option<Ident>],
    flatten: &Override<HashMap<Ident, ()>>,
    variant_key: &VariantKey,
) -> bool {
    let variant = &variant_key.variant;
    let meta_variant = variant_key.meta_variant.as_ref();
    match flatten {
        Override::Inherit => false,
        Override::Explicit(map) => {
            let contains_variant = map.contains_key(variant);
            let contains_meta_variant = meta_variant.map_or(true, |mv| map.contains_key(mv));

            let variants_exist = variant_names.iter().any(|v| map.contains_key(v));
            let meta_variants_exist = meta_variant_names
                .iter()
                .flatten()
                .any(|mv| map.contains_key(mv));

            if contains_variant && !meta_variants_exist {
                return false;
            }
            if contains_meta_variant && !variants_exist {
                return false;
            }

            let contains_all = contains_variant && contains_meta_variant;

            !map.is_empty() && !contains_all
        }
    }
}
