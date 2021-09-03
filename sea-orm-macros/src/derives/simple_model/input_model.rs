use bae::FromAttributes;
use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Attribute, Field, Result, Visibility};

use crate::util::{
    get_token_stream_attributes, get_token_stream_derives_with, has_attribute,
    option_type_to_inner_type, split_token_stream,
};

#[derive(Clone, FromAttributes)]
struct Input {
    derives: Option<TokenStream>,
    attrs: Option<TokenStream>,
    skip: Option<()>,
}

fn impl_active_model_trait(
    input_model_ident: Ident,
    entity_ident: Ident,
    fields: &[(Field, Option<Input>)],
) -> Result<TokenStream> {
    let get_fields = fields.iter().map(|(field, _)| {
        let field_name = field.ident.clone().unwrap();
        let column_name = format_ident!("{}", field_name.to_string().to_camel_case());

        if option_type_to_inner_type(&field.ty).is_some() || has_attribute("has_default", &field.attrs) {
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
        impl sea_orm::ActiveModelTrait for #input_model_ident {
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
                <#input_model_ident as std::default::Default>::default()
            }
        }
    );

    Ok(expanded)
}

pub(crate) fn expand_input_model(
    attrs: &[Attribute],
    vis: Visibility,
    ident: Ident,
    fields: Punctuated<Field, Comma>,
) -> Result<TokenStream> {
    let input_model_ident = format_ident!("{}Input", ident);
    let entity_ident = format_ident!("{}Entity", ident);
    let input_attrs = Input::try_from_attributes(attrs)?;

    let derives = input_attrs
        .clone()
        .and_then(|input_attrs| input_attrs.derives)
        .map(|derives_stream| {
            get_token_stream_derives_with(
                derives_stream,
                vec![quote!(Clone), quote!(Debug), quote!(Default)],
            )
        })
        .unwrap_or_else(|| quote!(#[derive(Clone, Debug, Default)]));

    let attributes = input_attrs
        .and_then(|input_attrs| input_attrs.attrs)
        .map(get_token_stream_attributes)
        .unwrap_or_default();

    let mut input_fields = Vec::new();
    for field in fields {
        if has_attribute("auto_identity", &field.attrs) {
            continue;
        }

        let attr = Input::try_from_attributes(&field.attrs)?;

        if attr
            .as_ref()
            .map(|attr| attr.skip.is_some())
            .unwrap_or_default()
        {
            continue;
        }

        input_fields.push((field, attr));
    }

    let input_field_attrs = input_fields.iter().map(|(_, attr)| {
        attr.as_ref()
            .and_then(|attr| {
                attr.attrs
                    .clone()
                    .map(|attrs| match split_token_stream(attrs, ',') {
                        Ok(field_attrs) => quote!(#(#[#field_attrs]) *),
                        Err(err) => err.to_compile_error(),
                    })
            })
            .unwrap_or_default()
    });

    let input_field_vis = input_fields.iter().map(|(field, _)| field.vis.clone());

    let input_field_names = input_fields
        .iter()
        .map(|(field, _)| field.ident.clone().unwrap());

    let input_field_types = input_fields.iter().map(|(field, _)| {
        let ty = field.ty.clone();
        if option_type_to_inner_type(&field.ty).is_none()
            && has_attribute("has_default", &field.attrs)
        {
            quote!(std::option::Option<#ty>)
        } else {
            quote!(#ty)
        }
    });

    let active_model_trait =
        impl_active_model_trait(input_model_ident.clone(), entity_ident, &input_fields)?;

    let expanded = quote!(
        #derives
        #attributes
        #vis struct #input_model_ident {
            #(#input_field_attrs #input_field_vis #input_field_names: #input_field_types),*
        }

        #active_model_trait
    );

    Ok(expanded)
}
