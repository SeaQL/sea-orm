use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed, Result};

use crate::derives::simple_model::input_model::expand_input_model;

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

    let input_model = expand_input_model(&attrs, vis, ident, fields)?;

    let expanded = quote!(
        #input_model
    );

    Ok(expanded)
}
