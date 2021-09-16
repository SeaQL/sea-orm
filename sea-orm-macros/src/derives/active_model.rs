use crate::util::field_not_ignored;
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
                ident.span() => compile_error!("you can only derive DeriveActiveModel on structs");
            })
        }
    };

    let fields = fields.into_iter().filter(field_not_ignored);

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
        #[derive(Clone, Debug, PartialEq)]
        pub struct ActiveModel {
            #(pub #field: sea_orm::ActiveValue<#ty>),*
        }

        impl std::default::Default for ActiveModel {
            fn default() -> Self {
                <Self as sea_orm::ActiveModelBehavior>::new()
            }
        }

        impl std::convert::From<<Entity as EntityTrait>::Model> for ActiveModel {
            fn from(m: <Entity as EntityTrait>::Model) -> Self {
                Self {
                    #(#field: sea_orm::unchanged_active_value_not_intended_for_public_use(m.#field)),*
                }
            }
        }

        impl sea_orm::IntoActiveModel<ActiveModel> for <Entity as EntityTrait>::Model {
            fn into_active_model(self) -> ActiveModel {
                self.into()
            }
        }

        impl sea_orm::ActiveModelTrait for ActiveModel {
            type Entity = Entity;

            fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => {
                        let mut value = sea_orm::ActiveValue::unset();
                        std::mem::swap(&mut value, &mut self.#field);
                        value.into_wrapped_value()
                    },)*
                    _ => sea_orm::ActiveValue::unset(),
                }
            }

            fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field.clone().into_wrapped_value(),)*
                    _ => sea_orm::ActiveValue::unset(),
                }
            }

            fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::set(v.unwrap()),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn unset(&mut self, c: <Self::Entity as EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::unset(),)*
                    _ => {},
                }
            }

            fn is_unset(&self, c: <Self::Entity as EntityTrait>::Column) -> bool {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field.is_unset(),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn default() -> Self {
                Self {
                    #(#field: sea_orm::ActiveValue::unset()),*
                }
            }
        }
    ))
}
