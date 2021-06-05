use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Data;

pub fn expand_derive_active_model_behavior(_ident: Ident, _data: Data) -> syn::Result<TokenStream> {
    Ok(quote!(
        impl sea_orm::ActiveModelBehavior for ActiveModel {
            type Entity = Entity;
        }
    ))
}
