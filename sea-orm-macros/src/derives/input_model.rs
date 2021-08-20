use bae::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, Fields, FieldsNamed};

use crate::util::option_type_to_inner_type;

#[derive(Debug, Default, Eq, PartialEq, FromAttributes)]
pub struct Sea {
    // Marks a non Option<T> field as optional
    optional: Option<()>,
    // Marks a field to be omitted from the InputModel
    skip_graphql: Option<()>,
}

pub fn expand_derive_input_model(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveActiveModel on structs");
            })
        }
    };

    let mut field_types = Vec::new();
    for field in fields {
        let sea_attr = Sea::try_from_attributes(&field.attrs)?.unwrap_or_default();
        if sea_attr.skip_graphql.is_some() {
            continue;
        }

        let inner_option_type_maybe = option_type_to_inner_type(&field.ty);
        let ty = inner_option_type_maybe
            .map(Clone::clone)
            .unwrap_or_else(|| field.ty.clone());
        let optional = sea_attr.optional.is_some() || inner_option_type_maybe.is_some();

        let ident = field.ident.expect("struct field is missing an ident");

        if optional {
            field_types.push(quote!(pub #ident: std::option::Option<#ty>))
        } else {
            field_types.push(quote!(pub #ident: #ty))
        }
    }

    Ok(quote!(
        #[derive(Clone, Default, Debug, PartialEq)]
        pub struct InputModel {
            #(#field_types),*
        }
    ))
}
