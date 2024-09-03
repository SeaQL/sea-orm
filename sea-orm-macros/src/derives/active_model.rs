use super::util::{
    escape_rust_keyword, field_not_ignored, format_field_ident, trim_starting_raw_identifier,
};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{punctuated::IntoIter, Data, DataStruct, Expr, Field, Fields, LitStr, Type};

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

    let derive_active_model = derive_active_model(&ident, all_fields.clone())?;
    let derive_into_model = derive_into_model(&ident, all_fields)?;

    Ok(quote!(
        #derive_active_model
        #derive_into_model
    ))
}

fn derive_active_model(ident: &Ident, all_fields: IntoIter<Field>) -> syn::Result<TokenStream> {
    let fields = all_fields.filter(field_not_ignored);

    let field: Vec<Ident> = fields.clone().map(format_field_ident).collect();

    let name: Vec<Ident> = fields
        .clone()
        .map(|field| {
            let ident = field.ident.as_ref().unwrap().to_string();
            let ident = trim_starting_raw_identifier(ident).to_upper_camel_case();
            let ident = escape_rust_keyword(ident);
            let mut ident = format_ident!("{}", &ident);
            field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("sea_orm"))
                .try_for_each(|attr| {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("enum_name") {
                            let litstr: LitStr = meta.value()?.parse()?;
                            ident = syn::parse_str(&litstr.value()).unwrap();
                        } else {
                            // Reads the value expression to advance the parse stream.
                            // Some parameters, such as `primary_key`, do not have any value,
                            // so ignoring an error occurred here.
                            let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
                        }

                        Ok(())
                    })
                })?;
            Ok::<Ident, syn::Error>(ident)
        })
        .collect::<Result<_, _>>()?;

    let ty: Vec<Type> = fields.into_iter().map(|Field { ty, .. }| ty).collect();

    Ok(quote!(
        #[doc = " Generated by sea-orm-macros"]
        #[derive(Clone, Debug, PartialEq)]
        pub struct ActiveModel {

            #(
                #[doc = " Generated by sea-orm-macros"]
                pub #field: sea_orm::ActiveValue<#ty>
            ),*
        }

        #[automatically_derived]
        impl std::default::Default for ActiveModel {
            fn default() -> Self {
                <Self as sea_orm::ActiveModelBehavior>::new()
            }
        }

        #[automatically_derived]
        impl std::convert::From<#ident> for ActiveModel {
            fn from(m: #ident) -> Self {
                Self {
                    #(#field: sea_orm::ActiveValue::unchanged(m.#field)),*
                }
            }
        }

        #[automatically_derived]
        impl sea_orm::IntoActiveModel<ActiveModel> for #ident {
            fn into_active_model(self) -> ActiveModel {
                self.into()
            }
        }

        #[automatically_derived]
        impl sea_orm::ActiveModelTrait for ActiveModel {
            type Entity = Entity;

            fn take(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#name => {
                        let mut value = sea_orm::ActiveValue::not_set();
                        std::mem::swap(&mut value, &mut self.#field);
                        value.into_wrapped_value()
                    },)*
                    _ => sea_orm::ActiveValue::not_set(),
                }
            }

            fn get(&self, c: <Self::Entity as sea_orm::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#name => self.#field.clone().into_wrapped_value(),)*
                    _ => sea_orm::ActiveValue::not_set(),
                }
            }

            fn set(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column, v: sea_orm::Value) {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::set(v.unwrap()),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn not_set(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#name => self.#field = sea_orm::ActiveValue::not_set(),)*
                    _ => {},
                }
            }

            fn is_not_set(&self, c: <Self::Entity as sea_orm::EntityTrait>::Column) -> bool {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#name => self.#field.is_not_set(),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }

            fn default() -> Self {
                Self {
                    #(#field: sea_orm::ActiveValue::not_set()),*
                }
            }

            fn reset(&mut self, c: <Self::Entity as sea_orm::EntityTrait>::Column) {
                match c {
                    #(<Self::Entity as sea_orm::EntityTrait>::Column::#name => self.#field.reset(),)*
                    _ => panic!("This ActiveModel does not have this field"),
                }
            }
        }
    ))
}

fn derive_into_model(ident: &Ident, model_fields: IntoIter<Field>) -> syn::Result<TokenStream> {
    let active_model_fields = model_fields.clone().filter(field_not_ignored);

    let active_model_field: Vec<Ident> = active_model_fields
        .into_iter()
        .map(format_field_ident)
        .collect();
    let model_field: Vec<Ident> = model_fields.clone().map(format_field_ident).collect();

    let ignore_attr: Vec<bool> = model_fields
        .map(|field| !field_not_ignored(&field))
        .collect();

    let model_field_value: Vec<TokenStream> = model_field
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
        #[automatically_derived]
        impl std::convert::TryFrom<ActiveModel> for #ident {
            type Error = sea_orm::DbErr;
            fn try_from(a: ActiveModel) -> Result<Self, sea_orm::DbErr> {
                #(if matches!(a.#active_model_field, sea_orm::ActiveValue::NotSet) {
                    return Err(sea_orm::DbErr::AttrNotSet(stringify!(#active_model_field).to_owned()));
                })*
                Ok(
                    Self {
                        #(#model_field: #model_field_value),*
                    }
                )
            }
        }

        #[automatically_derived]
        impl sea_orm::TryIntoModel<#ident> for ActiveModel {
            fn try_into_model(self) -> Result<#ident, sea_orm::DbErr> {
                self.try_into()
            }
        }
    ))
}
