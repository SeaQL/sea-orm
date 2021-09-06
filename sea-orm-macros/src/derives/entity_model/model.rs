use std::{borrow::Cow, iter::FromIterator};

use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::DeriveEntityModel;

pub struct Model<'a> {
    column_names: Vec<syn::Ident>,
    entity_ident: Cow<'a, syn::Ident>,
    field_names: Vec<&'a syn::Ident>,
    ident: &'a syn::Ident,
}

impl<'a> Model<'a> {
    pub fn from_entity_model(entity_model: &'a DeriveEntityModel) -> Self {
        let ident = &entity_model.ident;

        let entity_ident = entity_model
            .sea_attr
            .entity
            .as_ref()
            .map(|entity| Cow::Borrowed(entity))
            .unwrap_or_else(|| Cow::Owned(format_ident!("Entity")));

        let field_names: Vec<_> = entity_model
            .fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect();

        let column_names = field_names
            .iter()
            .map(|field_name| format_ident!("{}", field_name.to_string().to_camel_case()))
            .collect();

        Model {
            column_names,
            entity_ident,
            field_names,
            ident,
        }
    }

    pub fn expand(&self) -> TokenStream {
        let expanded_impl_from_query_result = self.impl_from_query_result();
        let expanded_impl_model_trait = self.impl_model_trait();

        TokenStream::from_iter([expanded_impl_from_query_result, expanded_impl_model_trait])
    }

    fn impl_from_query_result(&self) -> TokenStream {
        let ident = &self.ident;
        let field_names = &self.field_names;
        let column_names = &self.column_names;

        quote!(
            impl sea_orm::FromQueryResult for #ident {
                fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
                    Ok(Self {
                        #(#field_names: row.try_get(pre, sea_orm::IdenStatic::as_str(&<<Self as sea_orm::ModelTrait>::Entity as sea_orm::entity::EntityTrait>::Column::#column_names).into())?),*
                    })
                }
            }
        )
    }

    fn impl_model_trait(&self) -> TokenStream {
        let ident = &self.ident;
        let entity_ident = &self.entity_ident;
        let field_names = &self.field_names;
        let column_names = &self.column_names;

        let missing_field_msg = format!("field does not exist on {}", ident);

        quote!(
            impl sea_orm::ModelTrait for #ident {
                type Entity = #entity_ident;

                fn get(&self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> sea_orm::Value {
                    match c {
                        #(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_names => self.#field_names.clone().into(),)*
                        _ => panic!(#missing_field_msg),
                    }
                }

                fn set(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column, v: sea_orm::Value) {
                    match c {
                        #(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_names => self.#field_names = v.unwrap(),)*
                        _ => panic!(#missing_field_msg),
                    }
                }
            }
        )
    }
}
