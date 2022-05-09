use proc_macro2::Span;
use quote::quote;
use smallvec::{smallvec, SmallVec};
use syn::Token;

/// Convert an identifier from CamelCase to snake_case.
pub fn snake_case(ident: &str) -> String {
    ident
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            let chars: SmallVec<[char; 2]> = if c.is_uppercase() {
                if i == 0 {
                    c.to_lowercase().collect()
                } else {
                    std::iter::once('_').chain(c.to_lowercase()).collect()
                }
            } else {
                smallvec![c]
            };
            chars
        })
        .collect()
}

/// Create a generics block like `<_, _, _>` with `num_generics` underscores.
pub fn underscore_generics(num_generics: usize) -> proc_macro2::TokenStream {
    let underscore = Token![_](Span::call_site());
    let underscores = std::iter::repeat(quote! { #underscore }).take(num_generics);
    quote! { <#(#underscores),*> }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snake_case_correct() {
        assert_eq!(snake_case("BeaconBlock"), "beacon_block");
        assert_eq!(snake_case("SignedBeaconBlock"), "signed_beacon_block");
        assert_eq!(snake_case("StoreDHT"), "store_d_h_t"); // may want to change this in future
        assert_eq!(snake_case("hello_world"), "hello_world");
        assert_eq!(snake_case("__"), "__");
    }
}
