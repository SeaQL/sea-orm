use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, Field, Fields};

pub fn expand_derive_from_query_result(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => named.named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive FromQueryResult on structs");
            })
        }
    };

    let field: Vec<Ident> = fields
        .into_iter()
        .map(|Field { ident, .. }| format_ident!("{}", ident.unwrap().to_string()))
        .collect();

    let name: Vec<TokenStream> = field
        .iter()
        .map(|f| {
            let s = f.to_string();
            quote! { #s }
        })
        .collect();

    Ok(quote!(
        impl sea_orm::FromQueryResult for #ident {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
                Ok(Self {
                    #(#field: row.try_get(pre, #name)?),*
                })
            }
        }
    ))
}
