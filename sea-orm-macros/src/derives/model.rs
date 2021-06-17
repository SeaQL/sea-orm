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
            type Entity = Entity;

            fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::Value {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field.clone().into(),)*
                    _ => panic!("This Model does not have this field"),
                }
            }

            fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = v.unwrap(),)*
                    _ => panic!("This Model does not have this field"),
                }
            }
        }

        impl sea_orm::FromQueryResult for #ident {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::TypeErr> {
                Ok(Self {
                    #(#field: row.try_get(pre, <<Self as ModelTrait>::Entity as EntityTrait>::Column::#name.as_str().into())?),*
                })
            }
        }
    ))
}
