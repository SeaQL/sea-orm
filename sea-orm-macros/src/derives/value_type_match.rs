use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::{GenericArgument, Ident, LitStr, PathArguments, Type, TypePath};

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
    field_type: &Type,
    field_span: Span,
) -> Option<Ident> {
    let (nullable, field_type) = if let Type::Path(type_path) = field_type
        && let Some(inner) = generic_type_arg(type_path, "Option")
    {
        (true, inner)
    } else {
        (false, field_type)
    };

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

    let Type::Path(field_type) = field_type else {
        return None;
    };

    let value_type = if is_type(field_type, "bool") {
        Some("BoolColumn")
    } else if is_type(field_type, "String") {
        if nullable {
            Some("StringColumnNullable")
        } else {
            Some("StringColumn")
        }
    } else if let Some(inner) = generic_type_arg(field_type, "Vec") {
        if let Type::Path(inner) = inner {
            if is_type(inner, "u8") {
                Some("BytesColumn")
            } else if is_numeric_type(inner) {
                Some("NumericArrayColumn")
            } else {
                Some("GenericArrayColumn")
            }
        } else {
            Some("GenericArrayColumn")
        }
    } else if is_type(field_type, "Uuid") {
        Some("UuidColumn")
    } else if is_type(field_type, "IpNetwork") {
        Some("IpNetworkColumn")
    } else if is_type(field_type, "Json") || is_serde_json_value(field_type) {
        Some("JsonColumn")
    } else if is_type(field_type, "TextUuid") {
        Some("TextUuidColumn")
    } else if is_numeric_type(field_type) || type_ident_contains(field_type, "UnixTimestamp") {
        if nullable {
            Some("NumericColumnNullable")
        } else {
            Some("NumericColumn")
        }
    } else if type_ident_contains(field_type, "DateTime")
        || type_ident_contains(field_type, "Timestamp")
    {
        Some("DateTimeLikeColumn")
    } else if type_ident_contains(field_type, "Date") {
        Some("DateLikeColumn")
    } else if type_ident_contains(field_type, "Time") {
        Some("TimeLikeColumn")
    } else {
        None
    };

    value_type.map(|ty| Ident::new(ty, field_span))
}

fn generic_type_arg<'a>(type_path: &'a TypePath, ident: &str) -> Option<&'a Type> {
    let segment = type_path.path.segments.last()?;
    if segment.ident != ident {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(inner)) = args.args.first() else {
        return None;
    };
    Some(inner)
}

fn is_type(type_path: &TypePath, ident: &str) -> bool {
    type_path.path.segments.len() == 1
        && type_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == ident)
}

fn type_ident_contains(type_path: &TypePath, pattern: &str) -> bool {
    type_path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident.to_string().contains(pattern))
}

fn is_serde_json_value(type_path: &TypePath) -> bool {
    let mut segments = type_path.path.segments.iter();
    matches!(
        (segments.next(), segments.next(), segments.next()),
        (Some(first), Some(second), None)
            if first.ident == "serde_json" && second.ident == "Value"
    )
}

fn is_numeric_type(type_path: &TypePath) -> bool {
    if type_path.path.segments.len() != 1 {
        return false;
    }

    type_path.path.segments.last().is_some_and(|segment| {
        segment.ident == "i8"
            || segment.ident == "i16"
            || segment.ident == "i32"
            || segment.ident == "i64"
            || segment.ident == "u8"
            || segment.ident == "u16"
            || segment.ident == "u32"
            || segment.ident == "u64"
            || segment.ident == "f32"
            || segment.ident == "f64"
            || segment.ident == "Decimal"
            || segment.ident == "BigDecimal"
    })
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
