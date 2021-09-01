use bae::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Attribute, Field, Result, Visibility};

use crate::derives::simple_model::util::{
    get_token_stream_attributes, get_token_stream_derives, has_attribute, split_token_stream,
};

#[derive(Clone, FromAttributes)]
struct Input {
    derives: Option<TokenStream>,
    attrs: Option<TokenStream>,
}

pub(crate) fn expand_input_model(
    attrs: &[Attribute],
    vis: Visibility,
    ident: Ident,
    fields: Punctuated<Field, Comma>,
) -> Result<TokenStream> {
    let input_model_ident = format_ident!("{}Input", ident);
    let input_attrs = Input::try_from_attributes(attrs)?;

    let derives = input_attrs
        .clone()
        .and_then(|input_attrs| input_attrs.derives)
        .map(get_token_stream_derives)
        .unwrap_or_default();

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

        input_fields.push((field, attr));
    }

    let input_field_attrs: Vec<_> = input_fields
        .iter()
        .map(|(_, attr)| {
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
        })
        .collect();

    let input_field_vis: Vec<_> = input_fields
        .iter()
        .map(|(field, _)| field.vis.clone())
        .collect();

    let input_field_names: Vec<_> = input_fields
        .iter()
        .map(|(field, _)| field.ident.clone().unwrap())
        .collect();

    let input_field_types: Vec<_> = input_fields
        .iter()
        .map(|(field, _)| field.ty.clone())
        .collect();

    let expanded = quote!(
        #derives
        #attributes
        #vis struct #input_model_ident {
            #(#input_field_attrs #input_field_vis #input_field_names: #input_field_types),*
        }
    );

    Ok(expanded)
}
