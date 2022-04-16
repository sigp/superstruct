use crate::attributes::IdentList;
use crate::naming::generate_map_macro_name;
use crate::utils::underscore_generics;
use crate::{StructOpts, TokenStream, TokenStream2};
use quote::quote;
use syn::Ident;

pub struct MacroFromType<'a> {
    /// The name of the superstruct type being matched on.
    pub name: &'a Ident,
    /// The number of generic type parameters.
    pub num_generics: usize,
    /// The names of the variant structs.
    pub struct_names: &'a [Ident],
}

/// Generate all the map macros for the top-level enum, Ref and RefMut.
pub(crate) fn generate_all_map_macros(
    type_name: &Ident,
    ref_type_name: &Ident,
    ref_mut_type_name: &Ident,
    num_generics: usize,
    struct_names: &[Ident],
    variant_names: &[Ident],
    opts: &StructOpts,
    output_items: &mut Vec<TokenStream>,
) {
    generate_all_map_macros_for_type(
        type_name,
        num_generics,
        struct_names,
        &opts.map_into,
        |from_type, to_type| generate_owned_map_macro(from_type, to_type, variant_names),
        output_items,
    );

    generate_all_map_macros_for_type(
        ref_type_name,
        num_generics,
        struct_names,
        &opts.map_ref_into,
        |from_type, to_type| generate_ref_map_macro(from_type, to_type, variant_names, false),
        output_items,
    );

    generate_all_map_macros_for_type(
        ref_mut_type_name,
        num_generics,
        struct_names,
        &opts.map_ref_mut_into,
        |from_type, to_type| generate_ref_map_macro(from_type, to_type, variant_names, true),
        output_items,
    );
}

fn generate_all_map_macros_for_type(
    type_name: &Ident,
    num_generics: usize,
    struct_names: &[Ident],
    map_into: &Option<IdentList>,
    generator: impl Fn(&MacroFromType, Option<&Ident>) -> TokenStream2,
    output_items: &mut Vec<TokenStream>,
) {
    let from_type = MacroFromType {
        name: type_name,
        num_generics,
        struct_names,
    };
    let map_macro_self = generator(&from_type, None);
    output_items.push(map_macro_self.into());

    if let Some(map_into) = map_into {
        for to_type in &map_into.idents {
            let map_macro_to = generator(&from_type, Some(to_type));
            output_items.push(map_macro_to.into());
        }
    }
}

fn generate_owned_map_macro(
    from_type: &MacroFromType,
    to_type_name: Option<&Ident>,
    variant_names: &[Ident],
) -> TokenStream2 {
    assert_eq!(
        from_type.struct_names.len(),
        variant_names.len(),
        "there must be one struct per variant"
    );

    let from_type_name = &from_type.name;
    let from_type_struct_names = from_type.struct_names;
    let to_type_name = to_type_name.unwrap_or_else(|| from_type_name);
    let map_macro_name = generate_map_macro_name(from_type_name, to_type_name);

    // Generics we want the compiler to infer.
    let from_type_generics = underscore_generics(from_type.num_generics);

    quote! {
        #[macro_export]
        macro_rules! #map_macro_name {
            ($value:expr, $f:expr) => {
                match $value {
                    #(
                        #from_type_name::#variant_names(inner) => {
                            let f: fn(
                                #from_type_struct_names #from_type_generics,
                                fn(_) -> _,
                            ) -> _ = $f;
                            f(inner, #to_type_name::#variant_names)
                        }
                    )*
                }
            }
        }
    }
}

fn generate_ref_map_macro(
    from_type: &MacroFromType,
    to_type_name: Option<&Ident>,
    variant_names: &[Ident],
    mutable: bool,
) -> TokenStream2 {
    assert_eq!(
        from_type.struct_names.len(),
        variant_names.len(),
        "there must be one struct per variant"
    );

    let from_type_name = &from_type.name;
    let from_type_struct_names = from_type.struct_names;
    let to_type_name = to_type_name.unwrap_or_else(|| from_type_name);
    let map_macro_name = generate_map_macro_name(from_type_name, to_type_name);

    // Generics we want the compiler to infer.
    let from_type_generics = underscore_generics(from_type.num_generics);

    let mutability = if mutable {
        quote! { mut }
    } else {
        quote! {}
    };

    quote! {
        #[macro_export]
        macro_rules! #map_macro_name {
            (&$lifetime:tt _, $value:expr, $f:expr) => {
                match $value {
                    #(
                        #from_type_name::#variant_names(inner) => {
                            let f: fn(
                                &$lifetime #mutability #from_type_struct_names #from_type_generics,
                                fn(_) -> _,
                            ) -> _ = $f;
                            f(inner, #to_type_name::#variant_names)
                        }
                    )*
                }
            }
        }
    }
}
