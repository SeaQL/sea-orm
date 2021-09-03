use bae::FromAttributes;
use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field, Fields,
    FieldsNamed, Generics, Result,
};

use crate::{
    derives::simple_input::field_validation::expand_field_validation,
    util::option_type_to_inner_type,
};

mod field_validation;

#[derive(FromAttributes)]
struct Input {
    model: Ident,
    entity: Option<Ident>,
}

pub(crate) fn expand_derive_simple_input(input: DeriveInput) -> Result<TokenStream> {
    let input_attr = Input::from_attributes(&input.attrs)?;
    let model_ident = input_attr.model;
    let ident = input.ident;
    let entity_ident = input_attr
        .entity
        .unwrap_or_else(|| format_ident!("{}Entity", model_ident));

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive SimpleInput on structs");
            })
        }
    };

    let active_model_trait =
        impl_active_model_trait(&ident, input.generics, &entity_ident, &fields)?;
    let field_validation = expand_field_validation(&model_ident, &fields)?;

    let expanded = quote!(
        #active_model_trait

        #field_validation
    );

    Ok(expanded)
}

fn impl_active_model_trait(
    input_model_ident: &Ident,
    mut input_model_generics: Generics,
    entity_ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> Result<TokenStream> {
    input_model_generics
        .lifetimes_mut()
        .into_iter()
        .for_each(|mut lifetime| {
            lifetime.lifetime.ident = format_ident!("_");
        });

    let get_fields = fields.iter().map(|field| {
        let field_name = field.ident.clone().unwrap();
        let column_name = format_ident!("{}", field_name.to_string().to_camel_case());

        if option_type_to_inner_type(&field.ty).is_some() {
            quote!(
                <Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_name => {
                    if let Some(value) = &self.#field_name {
                        sea_orm::ActiveValue::set(value.clone()).into_wrapped_value()
                    } else {
                        sea_orm::ActiveValue::unset()
                    }
                }
            )
        } else {
            quote!(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_name => sea_orm::ActiveValue::set(self.#field_name.clone()).into_wrapped_value())
        }
    });

    let expanded = quote!(
        impl sea_orm::ActiveModelTrait for #input_model_ident#input_model_generics {
            type Entity = #entity_ident;

            fn take(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                self.get(c)
            }

            fn get(&self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(#get_fields,)*
                    _ => sea_orm::ActiveValue::unset(),
                }
            }

            fn set(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column, v: sea_orm::Value) {
                panic!("cannot set on an input model")
            }

            fn unset(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) {
                panic!("cannot unset on an input model")
            }

            fn is_unset(&self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> bool {
                panic!("cannot is_unset on an input model")
            }

            fn default() -> Self {
                <#input_model_ident#input_model_generics as std::default::Default>::default()
            }
        }
    );

    Ok(expanded)
}
