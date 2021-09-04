use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Field, Type};

use crate::util::option_type_to_inner_type;

pub(crate) fn expand_field_validation(
    model_ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let checks = fields.into_iter().map(|field| {
        let fn_name = format_ident!(
            "_Assert{}{}",
            model_ident,
            field.ident.as_ref().unwrap().to_string().to_camel_case()
        );

        let ty = {
            let mut ty = option_type_to_inner_type(&field.ty)
                .map(Clone::clone)
                .unwrap_or_else(|| field.ty.clone());

            // Rename lifetimes to _
            if let Type::Reference(ref mut type_ref) = ty {
                type_ref.lifetime = type_ref.lifetime.clone().map(|mut lifetime| {
                    lifetime.ident = format_ident!("_");
                    lifetime
                });
            }

            ty
        };

        quote_spanned!(field.ty.span()=> impl #fn_name<#ty> for () {})
    });

    quote!(
        #(#checks)*
    )
}
