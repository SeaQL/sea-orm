use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Field, Result};

pub(crate) fn expand_model(ident: Ident, fields: Punctuated<Field, Comma>) -> Result<TokenStream> {
    let missing_field_msg = format!("field does not exist on {}", ident);
    let entity_ident = format_ident!("{}Entity", ident);

    let field_names: Vec<_> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();

    let column_names: Vec<_> = field_names
        .iter()
        .map(|field_name| format_ident!("{}", field_name.to_string().to_camel_case()))
        .collect();

    // let field_types: Vec<_> = fields.iter().map(|field| field.ty.clone()).collect();

    let expanded = quote!(
        impl sea_orm::ModelTrait for #ident {
            type Entity = #entity_ident;

            fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::Value {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#column_names => self.#field_names.clone().into(),)*
                    _ => panic!(#missing_field_msg),
                }
            }

            fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#column_names => self.#field_names = v.unwrap(),)*
                    _ => panic!(#missing_field_msg),
                }
            }
        }

        impl sea_orm::FromQueryResult for #ident {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
                Ok(Self {
                    #(#field_names: row.try_get(pre, <<Self as ModelTrait>::Entity as EntityTrait>::Column::#column_names.as_str().into())?),*
                })
            }
        }
    );

    Ok(expanded)
}
