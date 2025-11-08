use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{Ident, LitStr, Type};

pub fn column_type_expr(
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

pub fn column_type_wrapper(
    column_type: &Option<String>,
    field_type: &str,
    field_span: Span,
) -> Option<Ident> {
    match column_type {
        Some(_) => {
            let field_type = if let Some((prefix, _)) = field_type.split_once('(') {
                prefix
            } else {
                field_type
            };
            match field_type {
                "String" | "Text" => Some("StringColumn"),
                "Blob" | "Binary" | "VarBinary" => Some("BytesColumn"),
                "TinyInteger" | "SmallInteger" | "Integer" | "BigInteger" | "TinyUnsigned"
                | "SmallUnsigned" | "Unsigned" | "BigUnsigned" | "Float" | "Double" | "Decimal"
                | "Money" => Some("NumericColumn"),
                "DateTime" | "Timestamp" | "TimestampWithTimeZone" => Some("DateTimeLikeColumn"),
                "Time" => Some("TimeLikeColumn"),
                "Date" => Some("DateLikeColumn"),
                "Boolean" => Some("BoolColumn"),
                "Json" | "JsonBinary" => Some("JsonColumn"),
                "Uuid" => Some("UuidColumn"),
                "Array" => Some("GenericArrayColumn"),
                _ => None,
            }
            .map(|ty| Ident::new(ty, field_span))
        }
        None => match trim_option(field_type) {
            "bool" => Some("BoolColumn"),
            "String" => Some("StringColumn"),
            "Vec<u8>" => Some("BytesColumn"),
            "Uuid" => Some("UuidColumn"),
            "IpNetwork" => Some("IpNetworkColumn"),
            "Json" | "serde_json::Value" => Some("JsonColumn"),
            field_type => {
                if is_numeric_column(field_type) {
                    Some("NumericColumn")
                } else if field_type.starts_with("Vec<") {
                    let field_type = &field_type["Vec<".len()..field_type.len() - 1];
                    if is_numeric_column(field_type) {
                        Some("NumericArrayColumn")
                    } else {
                        Some("GenericArrayColumn")
                    }
                } else if field_type.contains("DateTime") || field_type.contains("Timestamp") {
                    Some("DateTimeLikeColumn")
                } else if field_type.contains("Date") {
                    Some("DateLikeColumn")
                } else if field_type.contains("Time") {
                    Some("TimeLikeColumn")
                } else {
                    None
                }
            }
        }
        .map(|ty| Ident::new(ty, field_span)),
    }
}

fn is_numeric_column(ty: &str) -> bool {
    matches!(
        ty,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "f32"
            | "f64"
            | "Decimal"
            | "BigDecimal"
    )
}

pub fn array_type_expr(
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

fn trim_option(s: &str) -> &str {
    if s.starts_with("Option<") {
        &s["Option<".len()..s.len() - 1]
    } else {
        s
    }
}
