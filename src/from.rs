//! Generate `From` implementations to convert variants to the top-level enum.
use quote::quote;
use syn::{Ident, ImplGenerics, Lifetime, TypeGenerics, WhereClause};

pub fn generate_from_variant_trait_impl(
    type_name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
    variant_name: &Ident,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        impl #impl_generics From<#struct_name #ty_generics> for #type_name #ty_generics #where_clause {
            fn from(variant: #struct_name #ty_generics) -> Self {
                Self::#variant_name(variant)
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn generate_from_variant_trait_impl_for_ref(
    ref_ty_name: &Ident,
    ref_ty_lifetime: &Lifetime,
    ref_impl_generics: &ImplGenerics,
    ref_ty_generics: &TypeGenerics,
    ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
    variant_name: &Ident,
    struct_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        impl #ref_impl_generics From<&#ref_ty_lifetime #struct_name #ty_generics> for #ref_ty_name #ref_ty_generics #where_clause {
            fn from(variant: &#ref_ty_lifetime #struct_name #ty_generics) -> Self {
                Self::#variant_name(variant)
            }
        }
    }
}

pub fn generate_from_enum_trait_impl_for_ref(
    ty_name: &Ident,
    ty_generics: &TypeGenerics,
    ref_ty_name: &Ident,
    ref_ty_lifetime: &Lifetime,
    ref_impl_generics: &ImplGenerics,
    ref_ty_generics: &TypeGenerics,
    where_clause: &Option<&WhereClause>,
) -> proc_macro2::TokenStream {
    quote! {
        impl #ref_impl_generics From<&#ref_ty_lifetime #ty_name #ty_generics> for #ref_ty_name #ref_ty_generics #where_clause {
            fn from(ref_to_enum: &#ref_ty_lifetime #ty_name #ty_generics) -> Self {
                ref_to_enum.to_ref()
            }
        }
    }
}
