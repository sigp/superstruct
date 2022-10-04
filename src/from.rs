//! Generate `From` implementations to convert variants to the top-level enum.
use quote::quote;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

pub fn generate_from_trait_impl(
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
