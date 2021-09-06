use std::iter::FromIterator;

use bae::FromAttributes;
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};

#[derive(Default, FromAttributes)]
pub struct Sea {
    pub entity: Option<syn::Ident>,
}

enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

struct DeriveModel {
    column_idents: Vec<syn::Ident>,
    entity_ident: syn::Ident,
    field_idents: Vec<syn::Ident>,
    ident: syn::Ident,
}

impl DeriveModel {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
                ..
            }) => named,
            _ => return Err(Error::InputNotStruct),
        };

        let sea_attr = Sea::try_from_attributes(&input.attrs)
            .map_err(Error::Syn)?
            .unwrap_or_default();

        let ident = input.ident;
        let entity_ident = sea_attr.entity.unwrap_or_else(|| format_ident!("Entity"));

        let field_idents = fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().clone())
            .collect();

        let column_idents = fields
            .iter()
            .map(|field| {
                format_ident!(
                    "{}",
                    field.ident.as_ref().unwrap().to_string().to_camel_case()
                )
            })
            .collect();

        Ok(DeriveModel {
            column_idents,
            entity_ident,
            field_idents,
            ident,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_from_query_result = self.impl_from_query_result();
        let expanded_impl_model_trait = self.impl_model_trait();

        Ok(TokenStream::from_iter([
            expanded_impl_from_query_result,
            expanded_impl_model_trait,
        ]))
    }

    fn impl_from_query_result(&self) -> TokenStream {
        let ident = &self.ident;
        let field_idents = &self.field_idents;
        let column_idents = &self.column_idents;

        quote!(
            impl sea_orm::FromQueryResult for #ident {
                fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
                    Ok(Self {
                        #(#field_idents: row.try_get(pre, sea_orm::IdenStatic::as_str(&<<Self as sea_orm::ModelTrait>::Entity as sea_orm::entity::EntityTrait>::Column::#column_idents).into())?),*
                    })
                }
            }
        )
    }

    fn impl_model_trait(&self) -> TokenStream {
        let ident = &self.ident;
        let entity_ident = &self.entity_ident;
        let field_idents = &self.field_idents;
        let column_idents = &self.column_idents;

        let missing_field_msg = format!("field does not exist on {}", ident);

        quote!(
            impl sea_orm::ModelTrait for #ident {
                type Entity = #entity_ident;

                fn get(&self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> sea_orm::Value {
                    match c {
                        #(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_idents => self.#field_idents.clone().into(),)*
                        _ => panic!(#missing_field_msg),
                    }
                }

                fn set(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column, v: sea_orm::Value) {
                    match c {
                        #(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_idents => self.#field_idents = v.unwrap(),)*
                        _ => panic!(#missing_field_msg),
                    }
                }
            }
        )
    }
}

pub fn expand_derive_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveModel::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
