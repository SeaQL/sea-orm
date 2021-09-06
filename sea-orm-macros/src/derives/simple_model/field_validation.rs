use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Field};

pub(crate) fn expand_field_validation(
    ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let fn_names: Vec<_> = fields
        .iter()
        .map(|field| {
            format_ident!(
                "_Assert{}{}",
                ident,
                field.ident.as_ref().unwrap().to_string().to_camel_case()
            )
        })
        .collect();

    let field_inner_types = fields.iter().map(|field| &field.ty);

    quote!(
        #(trait #fn_names<T: Into<#field_inner_types>> {})*
    )
}
