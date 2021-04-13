use darling::FromMeta;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::iter::{self, FromIterator};
use syn::{
    parse_macro_input, Attribute, AttributeArgs, Expr, GenericParam, Ident, ItemStruct, Lifetime,
    LifetimeDef, NestedMeta, Type, TypeGenerics,
};

/// Top-level configuration via the `superstruct` attribute.
#[derive(Debug, FromMeta)]
struct StructOpts {
    /// List of variant names of the superstruct being derived.
    variants: HashMap<Ident, ()>,
    /// List of attributes to apply to the variant structs.
    #[darling(default)]
    variant_attributes: Option<NestedMetaList>,
    /// List of attributes to apply to the generated Ref type.
    #[darling(default)]
    ref_attributes: Option<NestedMetaList>,
    /// List of attributes to apply to the generated MutRef type.
    #[darling(default)]
    ref_mut_attributes: Option<NestedMetaList>,
    /// Error type and expression to use for casting methods.
    #[darling(default)]
    cast_error: CastErrOpts,
}

/// Field-level configuration.
#[derive(Debug, Default, FromMeta)]
struct FieldOpts {
    #[darling(default)]
    only: Option<HashMap<Ident, ()>>,
    #[darling(default)]
    getter: Option<GetterOpts>,
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
struct CastErrOpts {
    #[darling(default)]
    ty: Option<String>,
    #[darling(default)]
    expr: Option<String>,
}

#[derive(Debug)]
struct NestedMetaList {
    metas: Vec<NestedMeta>,
}

impl FromMeta for NestedMetaList {
    fn from_list(items: &[NestedMeta]) -> Result<Self, darling::Error> {
        Ok(Self {
            metas: items.iter().cloned().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn superstruct(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as ItemStruct);

    let type_name = &item.ident;
    let visibility = item.vis;
    // Extract the generics to use for the top-level type and all variant structs.
    let decl_generics = &item.generics;
    // Generics used for the impl block.
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    let opts = StructOpts::from_list(&attr_args).unwrap();

    let mut output_items: Vec<TokenStream> = vec![];

    let mk_struct_name = |variant_name: &Ident| format_ident!("{}{}", type_name, variant_name);

    let variant_names = opts.variants.keys().cloned().collect_vec();
    let struct_names = variant_names.iter().map(mk_struct_name).collect_vec();

    // Vec of common fields, and getter options for them.
    let mut common_fields = vec![];
    // Map from variant to variant fields.
    let mut variant_fields =
        HashMap::<_, _>::from_iter(variant_names.iter().zip(iter::repeat(vec![])));

    for field in item.fields.iter() {
        let field_opts = field
            .attrs
            .iter()
            .filter(|attr| is_superstruct_attr(attr))
            .find_map(|attr| {
                let meta = attr.parse_meta().unwrap();
                Some(FieldOpts::from_meta(&meta).unwrap())
            })
            .unwrap_or_default();

        // Drop the field-level superstruct attributes
        let mut output_field = field.clone();
        output_field.attrs = filter_attributes(&output_field.attrs);

        // Add the field to the `variant_fields` map for all applicable variants.
        let field_variants = field_opts.only.as_ref().map_or_else(
            || variant_names.clone(),
            |only| only.keys().cloned().collect_vec(),
        );

        for variant_name in field_variants {
            variant_fields
                .get_mut(&variant_name)
                .expect("invalid variant name in `only`")
                .push(output_field.clone());
        }

        // Add to `common_fields`, including getter info.
        if field_opts.only.is_none() {
            common_fields.push((output_field.clone(), field_opts.getter.unwrap_or_default()));
        } else if field_opts.getter.is_some() {
            panic!("can't configure `only` and `getter` on the same field");
        }
    }

    // Generate structs for all of the variants.
    let struct_attributes = opts
        .variant_attributes
        .as_ref()
        .map_or(&[][..], |attrs| &attrs.metas);

    for (variant_name, struct_name) in variant_names.iter().zip(struct_names.iter()) {
        let fields = &variant_fields[variant_name];
        let variant_code = quote! {
            #(
                #[#struct_attributes]
            )*
            #visibility struct #struct_name #decl_generics #where_clause {
                #(
                    #fields,
                )*
            }
        };
        output_items.push(variant_code.into());
    }

    // Construct the top-level enum.
    let top_level_attrs = filter_attributes(&item.attrs);
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
        GenericParam::Lifetime(LifetimeDef::new(ref_ty_lifetime.clone())),
    );
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
        GenericParam::Lifetime(LifetimeDef::new(ref_mut_ty_lifetime.clone())),
    );
    let (ref_mut_impl_generics, ref_mut_ty_generics, _) = &ref_mut_ty_decl_generics.split_for_impl();

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
    let getters = common_fields.iter().map(|(field, getter_opts)| {
        let field_name = field.ident.as_ref().expect("named fields only");
        make_field_getter(
            type_name,
            &variant_names,
            field_name,
            &field.ty,
            None,
            getter_opts,
        )
    });

    let mut_getters = common_fields
        .iter()
        .filter(|(_, getter_opts)| !getter_opts.no_mut)
        .map(|(field, getter_opts)| {
            let field_name = field.ident.as_ref().expect("named fields only");
            make_mut_field_getter(
                type_name,
                &variant_names,
                field_name,
                &field.ty,
                None,
                getter_opts,
            )
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
        }
    };
    output_items.push(impl_block.into());

    // Construct the impl block for the *Ref type.
    let ref_getters = common_fields.iter().map(|(field, getter_opts)| {
        let field_name = field.ident.as_ref().expect("named fields only");
        make_field_getter(
            &ref_ty_name,
            &variant_names,
            field_name,
            &field.ty,
            Some(&ref_ty_lifetime),
            getter_opts,
        )
    });

    let ref_impl_block = quote! {
        impl #ref_impl_generics #ref_ty_name #ref_ty_generics #where_clause {
            #(
                #ref_getters
            )*
        }

        // Reference types are just wrappers around references, so they can be copied!
        impl #ref_impl_generics Copy for #ref_ty_name #ref_ty_generics #where_clause { }
        impl #ref_impl_generics Clone for #ref_ty_name #ref_ty_generics #where_clause {
            fn clone(&self) -> Self { *self }
        }
    };
    output_items.push(ref_impl_block.into());

    // Construct the impl block for the *MutRef type.
    let ref_mut_getters = common_fields.iter()
        .filter(|(_, getter_opts)| !getter_opts.no_mut)
        .map(|(field, getter_opts)| {
        let field_name = field.ident.as_ref().expect("named fields only");
        make_mut_field_getter(
            &ref_mut_ty_name,
            &variant_names,
            field_name,
            &field.ty,
            Some(&ref_mut_ty_lifetime),
            getter_opts,
        )
    });

    let ref_mut_impl_block = quote! {
        impl #ref_mut_impl_generics #ref_mut_ty_name #ref_mut_ty_generics #where_clause {
            #(
                #ref_mut_getters
            )*
        }
    };
    output_items.push(ref_mut_impl_block.into());

    TokenStream::from_iter(output_items)
}

/// Generate a getter method for a field.
fn make_field_getter(
    type_name: &Ident,
    variant_names: &[Ident],
    field_name: &Ident,
    field_type: &Type,
    lifetime: Option<&Lifetime>,
    getter_opts: &GetterOpts,
) -> proc_macro2::TokenStream {
    let fn_name = getter_opts.rename.as_ref().unwrap_or(field_name);
    let return_type = if getter_opts.copy {
        quote! { #field_type }
    } else {
        if let Some(lifetime) = lifetime {
            quote! { &#lifetime #field_type}
        } else {
            quote! { &#field_type}
        }
    };
    let return_expr = if getter_opts.copy {
        quote! { inner.#field_name }
    } else {
        quote! { &inner.#field_name }
    };
    quote! {
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
    field_name: &Ident,
    field_type: &Type,
    lifetime: Option<&Lifetime>,
    getter_opts: &GetterOpts,
) -> proc_macro2::TokenStream {
    let fn_name = format_ident!("{}_mut", getter_opts.rename.as_ref().unwrap_or(field_name));
    let return_type= if let Some(lifetime) = lifetime {
        quote! { &#lifetime mut #field_type}
    } else {
        quote! { &mut #field_type}
    };
    let param= if let Some(lifetime) = lifetime {
        quote! { &#lifetime mut self}
    } else {
        quote! { &mut self}
    };
    let return_expr = quote! { &mut inner.#field_name };
    quote! {
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

/// Generate a `as_<variant_name>{_mut}` method.
fn make_as_variant_method(
    type_name: &Ident,
    variant_name: &Ident,
    type_generics: &TypeGenerics,
    cast_err_opts: &CastErrOpts,
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
    let (ret_res_ty, err_expr) = if let Some(ref err_ty_str) = cast_err_opts.ty {
        let err_ty: Type = syn::parse_str(err_ty_str).expect("cast_error type not valid");
        let err_expr_str = cast_err_opts
            .expr
            .as_ref()
            .expect("must provide a cast_error(expr(..)) with ty");
        let err_expr: Expr = syn::parse_str(err_expr_str).expect("cast_error expr not valid");
        (quote! { Result<#ret_ty, #err_ty> }, quote! { #err_expr })
    } else {
        (quote! { Result<#ret_ty, ()> }, quote! { () })
    };
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

/// Keep all non-superstruct-related attributes from an array.
fn filter_attributes(attrs: &[Attribute]) -> Vec<Attribute> {
    attrs
        .iter()
        .filter(|attr| !is_superstruct_attr(attr))
        .cloned()
        .collect()
}

/// Predicate for determining whether an attribute is a `superstruct` attribute.
fn is_superstruct_attr(attr: &Attribute) -> bool {
    attr.path
        .get_ident()
        .map_or(false, |ident| ident.to_string() == "superstruct")
}
