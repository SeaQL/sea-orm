use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Result, Visibility};

pub(crate) fn expand_relation(vis: &Visibility, ident: &Ident) -> Result<TokenStream> {
    let relation_ident = format_ident!("{}Relation", ident);

    let expanded = quote!(
        #[derive(Copy, Clone, Debug, sea_orm::sea_strum::EnumIter)]
        #vis enum #relation_ident {}

        impl sea_orm::entity::RelationTrait for #relation_ident {
            fn def(&self) -> sea_orm::entity::RelationDef {
                panic!("No RelationDef")
            }
        }
    );

    Ok(expanded)
}
