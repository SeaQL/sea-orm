use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{LitStr, Type};

pub fn column_type_match(
    column_type: Option<TokenStream>,
    field_type: &str,
    field_span: Span,
) -> TokenStream {
    match column_type {
        Some(column_type) => {
            quote_spanned! { field_span => sea_orm::prelude::ColumnType::#column_type }
        }
        None => {
            let ty: Type = LitStr::new(field_type, field_span)
                .parse()
                .expect("field type error");
            quote_spanned! { field_span => <#ty as sea_orm::sea_query::ValueType>::column_type() }
        }
    }
}

pub fn array_type_match(
    array_type: Option<TokenStream>,
    field_type: &str,
    field_span: Span,
) -> TokenStream {
    match array_type {
        Some(array_type) => {
            quote_spanned! { field_span => sea_orm::sea_query::ArrayType::#array_type }
        }
        None => {
            let ty: Type = LitStr::new(field_type, field_span)
                .parse()
                .expect("field type error");
            quote_spanned! { field_span => <#ty as sea_orm::sea_query::ValueType>::array_type() }
        }
    }
}

pub fn can_try_from_u64(field_type: &str) -> bool {
    matches!(
        field_type,
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64"
    )
}
