use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Result};

mod column;
mod entity;
mod field_validation;
mod model;
mod primary_key;
mod relation;

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

    let entity = entity::expand_entity(&attrs, &vis, &ident)?;
    let column = column::expand_column(&vis, &ident, &fields);
    let primary_key = primary_key::expand_primary_key(&vis, &ident, &fields)?;
    let relation = relation::expand_relation(&vis, &ident);
    let model = model::expand_model(&ident, &fields);
    let field_validation = field_validation::expand_field_validation(&ident, &fields);

    Ok(quote!(
        #entity
        #column
        #primary_key
        #relation
        #model
        #field_validation
    ))
}
