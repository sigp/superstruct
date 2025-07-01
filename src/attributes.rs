//! Utilities to help with parsing configuration attributes.
use darling::{export::NestedMeta, Error, FromMeta};
use syn::Ident;

/// Parse a list of nested meta items.
///
/// Useful for passing through attributes intended for other macros.
#[derive(Debug)]
pub struct NestedMetaList {
    pub metas: Vec<NestedMeta>,
}

impl FromMeta for NestedMetaList {
    fn from_list(items: &[NestedMeta]) -> Result<Self, Error> {
        Ok(Self {
            metas: items.to_vec(),
        })
    }
}

/// List of identifiers implementing `FromMeta`.
///
/// Useful for imposing ordering, unlike the `HashMap` options provided by `darling`.
#[derive(Debug, Default, Clone)]
pub struct IdentList {
    pub idents: Vec<Ident>,
}

impl FromMeta for IdentList {
    fn from_list(items: &[NestedMeta]) -> Result<Self, Error> {
        let idents = items
            .iter()
            .map(|nested_meta| {
                let meta = match nested_meta {
                    NestedMeta::Meta(m) => m,
                    NestedMeta::Lit(l) => {
                        return Err(Error::custom(format!("expected ident, got literal: {l:?}")))
                    }
                };
                let path = meta.path();
                path.get_ident()
                    .cloned()
                    .ok_or(Error::custom(format!("can't parse as ident: {path:?}")))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { idents })
    }
}
