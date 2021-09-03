use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Result};

use crate::derives::simple_model::{
    column::expand_column, entity::expand_entity, field_validation::expand_field_validation,
    input_model::expand_input_model, model::expand_model, primary_key::expand_primary_key,
    relation::expand_relation,
};

pub(crate) fn expand_simple_model(input: DeriveInput) -> Result<TokenStream> {
    let attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive SimpleModel on structs");
            })
        }
    };

    let entity = expand_entity(&attrs, vis.clone(), ident.clone())?;
    let column = expand_column(vis.clone(), ident.clone(), fields.clone())?;
    let primary_key = expand_primary_key(vis.clone(), ident.clone(), fields.clone())?;
    let relation = expand_relation(vis.clone(), ident.clone())?;
    let model = expand_model(ident.clone(), fields.clone())?;
    let field_validation = expand_field_validation(ident.clone(), fields.clone())?;
    // let input_model = expand_input_model(&attrs, vis, ident, fields)?;

    let expanded = quote!(
        #entity

        #column

        #primary_key

        #relation

        #model

        #field_validation

        // #input_model
    );

    Ok(expanded)
}
