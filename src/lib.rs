use attributes::{IdentList, NestedMetaList};
use darling::FromMeta;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use std::iter::{self, FromIterator};
use syn::{
    parse_macro_input, Attribute, AttributeArgs, Expr, Field, GenericParam, Ident, ItemStruct,
    Lifetime, LifetimeDef, Type, TypeGenerics,
};

mod attributes;

/// Top-level configuration via the `superstruct` attribute.
#[derive(Debug, FromMeta)]
struct StructOpts {
    /// List of variant names of the superstruct being derived.
    variants: IdentList,
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
    cast_error: ErrorOpts,
    /// Error type and expression to use for partial getter methods.
    #[darling(default)]
    partial_getter_error: ErrorOpts,
    /// Turn off the generation of the top-level enum that binds the variants together.
    #[darling(default)]
    no_enum: bool,
}

/// Field-level configuration.
#[derive(Debug, Default, FromMeta)]
struct FieldOpts {
    #[darling(default)]
    only: Option<HashMap<Ident, ()>>,
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
    only: Option<Vec<Ident>>,
    getter_opts: GetterOpts,
    partial_getter_opts: GetterOpts,
}

impl FieldData {
    fn is_common(&self) -> bool {
        self.only.is_none()
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

    let variant_names = &opts.variants.idents;
    let struct_names = variant_names.iter().map(mk_struct_name).collect_vec();

    // Vec of field data.
    let mut fields = vec![];
    // Map from variant to variant fields.
    let mut variant_fields =
        HashMap::<_, _>::from_iter(variant_names.iter().zip(iter::repeat(vec![])));

    for field in item.fields.iter() {
        let name = field.ident.clone().expect("named fields only");
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
        output_field.attrs = discard_superstruct_attrs(&output_field.attrs);

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

        // Check field opts
        if field_opts.only.is_some() && field_opts.getter.is_some() {
            panic!("can't configure `only` and `getter` on the same field");
        } else if field_opts.only.is_none() && field_opts.partial_getter.is_some() {
            panic!("can't set `partial_getter` options on common field");
        }

        let only = field_opts.only.map(|only| only.keys().cloned().collect());
        let getter_opts = field_opts.getter.unwrap_or_default();
        let partial_getter_opts = field_opts.partial_getter.unwrap_or_default();

        // Add to list of all fields
        fields.push(FieldData {
            name,
            field: output_field,
            only,
            getter_opts,
            partial_getter_opts,
        });
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

    // If the `no_enum` attribute is set, stop after generating variant structs.
    if opts.no_enum {
        return TokenStream::from_iter(output_items);
    }

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
        .map(|field_data| make_field_getter(type_name, &variant_names, &field_data, None));

    let mut_getters = fields
        .iter()
        .filter(|f| f.is_common() && !f.getter_opts.no_mut)
        .map(|field_data| make_mut_field_getter(type_name, &variant_names, &field_data, None));

    let partial_getters = fields
        .iter()
        .filter(|f| !f.is_common())
        .cartesian_product(&[false, true])
        .flat_map(|(field_data, mutability)| {
            let field_variants = field_data.only.as_ref()?;
            Some(make_partial_getter(
                type_name,
                &field_data,
                &field_variants,
                &opts.partial_getter_error,
                *mutability,
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
            &variant_names,
            &field_data,
            Some(&ref_ty_lifetime),
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
    let ref_mut_getters = fields
        .iter()
        .filter(|f| f.is_common() && !f.getter_opts.no_mut)
        .map(|field_data| {
            make_mut_field_getter(
                &ref_mut_ty_name,
                &variant_names,
                &field_data,
                Some(&ref_mut_ty_lifetime),
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
    field_data: &FieldData,
    lifetime: Option<&Lifetime>,
) -> proc_macro2::TokenStream {
    let field_name = &field_data.name;
    let field_type = &field_data.field.ty;
    let getter_opts = &field_data.getter_opts;

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
) -> proc_macro2::TokenStream {
    let field_name = &field_data.name;
    let field_type = &field_data.field.ty;
    let getter_opts = &field_data.getter_opts;

    let fn_name = format_ident!("{}_mut", getter_opts.rename.as_ref().unwrap_or(field_name));
    let return_type = if let Some(lifetime) = lifetime {
        quote! { &#lifetime mut #field_type}
    } else {
        quote! { &mut #field_type}
    };
    let param = if let Some(lifetime) = lifetime {
        quote! { &#lifetime mut self}
    } else {
        quote! { &mut self}
    };
    let return_expr = quote! { &mut inner.#field_name };

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

fn make_self_arg(mutable: bool) -> proc_macro2::TokenStream {
    if mutable {
        quote! { &mut self }
    } else {
        quote! { &self }
    }
}

fn make_type_ref(ty: &Type, mutable: bool, copy: bool) -> proc_macro2::TokenStream {
    // XXX: bit hacky, ignore `copy` if `mutable` is set
    if mutable {
        quote! { &mut #ty }
    } else if copy {
        quote! { #ty }
    } else {
        quote! { &#ty }
    }
}

/// Generate a partial getter method for a field.
fn make_partial_getter(
    type_name: &Ident,
    field_data: &FieldData,
    field_variants: &[Ident],
    error_opts: &ErrorOpts,
    mutable: bool,
) -> proc_macro2::TokenStream {
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
    let self_arg = make_self_arg(mutable);
    let ret_ty = make_type_ref(&field_data.field.ty, mutable, copy);
    let ret_expr = if mutable {
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
    attr.path
        .get_ident()
        .map_or(false, |attr_ident| attr_ident.to_string() == ident)
}
