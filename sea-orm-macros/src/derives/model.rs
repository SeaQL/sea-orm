use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, Field, Fields, Type};

pub fn expend_derive_model(ident: Ident, data: Data) -> syn::Result<TokenStream> {
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
        .clone()
        .into_iter()
        .map(|Field { ident, .. }| format_ident!("{}", ident.unwrap().to_string().to_camel_case()))
        .collect();

    let ty: Vec<Type> = fields
        .into_iter()
        .map(|Field { ty, .. }| ty)
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

        #[derive(Clone, Debug)]
        pub struct ActiveModel {
            #(pub #field: sea_orm::Action<#ty>),*
        }

        impl sea_orm::ActiveModelOf<#ident> for ActiveModel {
            fn from_model(m: #ident) -> Self {
                Self::from(m)
            }
        }

        impl From<#ident> for ActiveModel {
            fn from(m: #ident) -> Self {
                Self {
                    #(#field: sea_orm::Action::Set(m.#field)),*
                }
            }
        }

        impl sea_orm::ActiveModelTrait for ActiveModel {
            type Column = Column;

            fn get(&self, c: Self::Column) -> sea_orm::Action<sea_orm::Value> {
                match c {
                    #(Self::Column::#name => self.#field.clone().into_action_value()),*
                }
            }

            fn set(&mut self, c: Self::Column, v: sea_orm::Value) {
                match c {
                    #(Self::Column::#name => self.#field = sea_orm::Action::Set(v.unwrap())),*
                }
            }

            fn unset(&mut self, c: Self::Column) {
                match c {
                    #(Self::Column::#name => self.#field = sea_orm::Action::Unset),*
                }
            }
        }
    ))
}
