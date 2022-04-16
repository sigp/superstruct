use crate::utils::snake_case;
use quote::format_ident;
use syn::Ident;

pub fn generate_map_macro_name(from_type_name: &Ident, to_type_name: &Ident) -> Ident {
    if from_type_name == to_type_name {
        format_ident!("map_{}", snake_case(&from_type_name.to_string()))
    } else {
        format_ident!(
            "map_{}_into_{}",
            snake_case(&from_type_name.to_string()),
            snake_case(&to_type_name.to_string())
        )
    }
}
