use bae::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Result};

mod field_validation;
mod insertable;

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

    let insertable = insertable::impl_insertable(&ident, input.generics, &entity_ident, &fields);
    let field_validation = field_validation::expand_field_validation(&model_ident, &fields);

    Ok(quote!(
        #insertable
        #field_validation
    ))
}
