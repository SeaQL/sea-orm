use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, Field, Fields, Type};

pub fn expand_derive_active_model(ident: Ident, data: Data) -> syn::Result<TokenStream> {
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

    let ty: Vec<Type> = fields.into_iter().map(|Field { ty, .. }| ty).collect();

    Ok(quote!(
        #[derive(Clone, Debug, Default)]
        pub struct ActiveModel {
            #(pub #field: sea_orm::ActiveValue<#ty>),*
        }

        impl sea_orm::ActiveModelOf<Entity> for ActiveModel {}

        impl From<#ident> for ActiveModel {
            fn from(m: #ident) -> Self {
                Self {
                    #(#field: sea_orm::unchanged_active_value_not_intended_for_public_use(m.#field)),*
                }
            }
        }

        impl sea_orm::ActiveModelTrait for ActiveModel {
            type Entity = Entity;

            fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => std::mem::take(&mut self.#field).into_wrapped_value()),*
                }
            }

            fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field.clone().into_wrapped_value()),*
                }
            }

            fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::set(v.unwrap())),*
                }
            }

            fn unset(&mut self, c: <Self::Entity as EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::unset()),*
                }
            }
        }
    ))
}
