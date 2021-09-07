extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

mod attributes;
mod derives;

#[proc_macro_derive(DeriveEntity, attributes(sea_orm))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_entity(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DeriveEntityModel, attributes(sea_orm))]
pub fn derive_entity_model(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input as DeriveInput);

    if ident != "Model" {
        panic!("Struct name must be Model");
    }

    match derives::expand_derive_entity_model(data, attrs) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DerivePrimaryKey)]
pub fn derive_primary_key(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_primary_key(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveColumn)]
pub fn derive_column(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_column(&ident, &data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveCustomColumn)]
pub fn derive_custom_column(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_custom_column(&ident, &data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveModel, attributes(sea_orm))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derives::expand_derive_model(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[proc_macro_derive(DeriveActiveModel)]
pub fn derive_active_model(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_active_model(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(DeriveActiveModelBehavior)]
pub fn derive_active_model_behavior(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_active_model_behavior(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(FromQueryResult)]
pub fn derive_from_query_result(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    match derives::expand_derive_from_query_result(ident, data) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn test(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    quote::quote! (
        #[test]
        #(#attrs)*
        fn #name() #ret {
            let _ = ::env_logger::builder()
                .filter_level(::log::LevelFilter::Debug)
                .is_test(true)
                .try_init();
            crate::block_on!(async { #body })
        }
    )
    .into()
}
