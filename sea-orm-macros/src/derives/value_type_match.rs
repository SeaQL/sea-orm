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
    let nullable = trim_option(field_type).0;

    if let Some(column_type) = column_type {
        let column_type = if let Some((prefix, _)) = column_type.split_once('(') {
            prefix
        } else {
            column_type
        };
        let value_type = match column_type {
            "String" | "Text" => {
                if nullable {
                    Some("StringColumnNullable")
                } else {
                    Some("StringColumn")
                }
            }
            "Blob" | "Binary" | "VarBinary" => Some("BytesColumn"),
            "TinyInteger" | "SmallInteger" | "Integer" | "BigInteger" | "TinyUnsigned"
            | "SmallUnsigned" | "Unsigned" | "BigUnsigned" | "Float" | "Double" | "Decimal"
            | "Money" => {
                if nullable {
                    Some("NumericColumnNullable")
                } else {
                    Some("NumericColumn")
                }
            }
            "DateTime" | "Timestamp" | "TimestampWithTimeZone" => Some("DateTimeLikeColumn"),
            "Time" => Some("TimeLikeColumn"),
            "Date" => Some("DateLikeColumn"),
            "Boolean" => Some("BoolColumn"),
            "Json" | "JsonBinary" => Some("JsonColumn"),
            "Uuid" => Some("UuidColumn"),
            "Array" => Some("GenericArrayColumn"),
            _ => None,
        }
        .map(|ty| Ident::new(ty, field_span));

        if value_type.is_some() {
            return value_type;
        }
    }

    match trim_option(field_type).1 {
        "bool" => Some("BoolColumn"),
        "String" => {
            if nullable {
                Some("StringColumnNullable")
            } else {
                Some("StringColumn")
            }
        }
        "Vec<u8>" => Some("BytesColumn"),
        "Uuid" => Some("UuidColumn"),
        "IpNetwork" => Some("IpNetworkColumn"),
        "Json" | "serde_json::Value" => Some("JsonColumn"),
        "TextUuid" => Some("TextUuidColumn"),
        field_type => {
            if is_numeric_column(field_type) || field_type.contains("UnixTimestamp") {
                if nullable {
                    Some("NumericColumnNullable")
                } else {
                    Some("NumericColumn")
                }
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
    .map(|ty| Ident::new(ty, field_span))
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

/// Return whether it is nullable
fn trim_option(s: &str) -> (bool, &str) {
    if s.starts_with("Option<") {
        (true, &s["Option<".len()..s.len() - 1])
    } else {
        (false, s)
    }
}
