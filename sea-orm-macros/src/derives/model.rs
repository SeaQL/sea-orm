use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, Field, Fields};

pub fn expand_derive_model(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => named.named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveModel on structs");
            })
        }
    };

    let field: Vec<Ident> = fields
        .clone()
        .into_iter()
        .map(|Field { ident, .. }| format_ident!("{}", ident.unwrap().to_string()))
        .collect();

    let name: Vec<Ident> = fields
        .into_iter()
        .map(|Field { ident, .. }| format_ident!("{}", ident.unwrap().to_string().to_camel_case()))
        .collect();

    Ok(quote!(
        impl sea_orm::ModelTrait for #ident {
            type Column = Column;

            fn get(&self, c: Self::Column) -> sea_orm::Value {
                match c {
                    #(Self::Column::#name => self.#field.clone().into()),*
                }
            }

            fn set(&mut self, c: Self::Column, v: sea_orm::Value) {
                match c {
                    #(Self::Column::#name => self.#field = v.unwrap()),*
                }
            }

            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::TypeErr> {
                Ok(Self {
                    #(#field: row.try_get(pre, Self::Column::#name.as_str().into())?),*
                })
            }
        }
    ))
}
