use bae::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Result};

use crate::util;

mod updatable;

#[derive(FromAttributes)]
struct Update {
    model: Ident,
    entity: Option<Ident>,
}

pub(crate) fn expand_derive_simple_update(input: DeriveInput) -> Result<TokenStream> {
    let input_attr = Update::from_attributes(&input.attrs)?;
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
                ident.span() => compile_error!("you can only derive SimpleUpdate on structs");
            })
        }
    };

    let updatable =
        updatable::impl_updatable(&ident, input.generics.clone(), &entity_ident, &fields);
    let field_validation =
        util::expand_model_field_validation(&ident, input.generics, &model_ident, &fields);

    Ok(quote!(
        #updatable
        #field_validation
    ))
}
