use crate::util::{escape_rust_keyword, field_not_ignored, trim_starting_raw_identifier};
use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{punctuated::Punctuated, token::Comma, Data, DataStruct, Field, Fields, Lit, Meta, Type};

/// Method to derive an [ActiveModel](sea_orm::ActiveModel)
pub fn expand_derive_active_model(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    // including ignored fields
    let all_fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => named.named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveActiveModel on structs");
            })
        }
    }
    .into_iter();

    let fields = all_fields.clone().filter(field_not_ignored);

    let all_field: Vec<Ident> = all_fields
        .clone()
        .into_iter()
        .map(|Field { ident, .. }| format_ident!("{}", ident.unwrap().to_string()))
        .collect();

    let field: Vec<Ident> = fields
        .clone()
        .into_iter()
        .map(|Field { ident, .. }| format_ident!("{}", ident.unwrap().to_string()))
        .collect();

    let name: Vec<Ident> = fields
        .clone()
        .into_iter()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap().to_string();
            let ident = trim_starting_raw_identifier(ident).to_camel_case();
            let ident = escape_rust_keyword(ident);
            let mut ident = format_ident!("{}", &ident);
            for attr in field.attrs.iter() {
                if let Some(ident) = attr.path.get_ident() {
                    if ident != "sea_orm" {
                        continue;
                    }
                } else {
                    continue;
                }
                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                {
                    for meta in list.iter() {
                        if let Meta::NameValue(nv) = meta {
                            if let Some(name) = nv.path.get_ident() {
                                if name == "enum_name" {
                                    if let Lit::Str(litstr) = &nv.lit {
                                        ident = syn::parse_str(&litstr.value()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ident
        })
        .collect();

    let ty: Vec<Type> = fields.into_iter().map(|Field { ty, .. }| ty).collect();

    let ignore_attr: Vec<bool> = all_fields
        .clone()
        .map(|field| !field_not_ignored(&field))
        .collect();

    let field_value: Vec<TokenStream> = all_field
        .iter()
        .zip(ignore_attr)
        .map(|(field, ignore)| {
            if ignore {
                quote! {
                Default::default()
                }
            } else {
                quote! {
                a.#field.into_value().unwrap().unwrap()
                }
            }
        })
        .collect();

    Ok(quote!(
        #[derive(Clone, Debug, PartialEq)]
        pub struct ActiveModel {
            #(pub #field: sea_orm::ActiveValue<#ty>),*
        }

        #[automatically_derived]
        impl std::default::Default for ActiveModel {
            fn default() -> Self {
                <Self as sea_orm::ActiveModelBehavior>::new()
            }
        }

        #[automatically_derived]
        impl std::convert::From<<Entity as EntityTrait>::Model> for ActiveModel {
            fn from(m: <Entity as EntityTrait>::Model) -> Self {
                Self {
                    #(#field: sea_orm::ActiveValue::unchanged(m.#field)),*
                }
            }
        }

        #[automatically_derived]
        impl sea_orm::IntoActiveModel<ActiveModel> for <Entity as EntityTrait>::Model {
            fn into_active_model(self) -> ActiveModel {
                self.into()
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<ActiveModel> for <Entity as EntityTrait>::Model {
            type Error = DbErr;
            fn try_from(a: ActiveModel) -> Result<Self, DbErr> {
                #(if matches!(a.#field, sea_orm::ActiveValue::NotSet) {
                    return Err(DbErr::Custom(format!("field {} is NotSet", stringify!(#field))));
                })*
                Ok(
                    Self {
                        #(#all_field: #field_value),*
                    }
                )
            }
        }

        #[automatically_derived]
        impl sea_orm::TryIntoModel<<Entity as EntityTrait>::Model> for ActiveModel {
            fn try_into_model(self) -> Result<<Entity as EntityTrait>::Model, DbErr> {
                self.try_into()
            }
        }

        #[automatically_derived]
        impl sea_orm::ActiveModelTrait for ActiveModel {
            type Entity = Entity;

            fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => {
                        let mut value = sea_orm::ActiveValue::not_set();
                        std::mem::swap(&mut value, &mut self.#field);
                        value.into_wrapped_value()
                    },)*
                    _ => sea_orm::ActiveValue::not_set(),
                }
            }

            fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field.clone().into_wrapped_value(),)*
                    _ => sea_orm::ActiveValue::not_set(),
                }
            }

            fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::set(v.unwrap()),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn not_set(&mut self, c: <Self::Entity as EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::not_set(),)*
                    _ => {},
                }
            }

            fn is_not_set(&self, c: <Self::Entity as EntityTrait>::Column) -> bool {
                match c {
                    #(<Self::Entity as EntityTrait>::Column::#name => self.#field.is_not_set(),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn default() -> Self {
                Self {
                    #(#field: sea_orm::ActiveValue::not_set()),*
                }
            }
        }
    ))
}
