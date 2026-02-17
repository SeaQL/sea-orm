use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, Fields, LitInt, LitStr, Type};

/// Expand the DeriveArrowSchema derive macro
pub fn expand_derive_arrow_schema(
    _ident: Ident,
    data: Data,
    _attrs: Vec<Attribute>,
) -> syn::Result<TokenStream> {
    if !cfg!(feature = "with-arrow") {
        return Ok(quote!());
    }

    let mut fields_info = Vec::new();

    // Parse fields
    if let Data::Struct(item_struct) = data {
        if let Fields::Named(fields) = &item_struct.fields {
            for field in &fields.named {
                if let Some(field_ident) = &field.ident {
                    let field_name = field_ident.to_string();
                    let field_type = &field.ty;

                    // Detect if field is Option<T> for nullability
                    let type_string = quote! { #field_type }.to_string().replace(' ', "");
                    let is_nullable = type_string.starts_with("Option<");

                    // Parse field attributes
                    let mut arrow_attrs = ArrowFieldAttrs::default();
                    let mut column_type_str: Option<String> = None;
                    let mut skip = false;

                    for attr in field.attrs.iter() {
                        if attr.path().is_ident("sea_orm") {
                            attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("arrow_skip") {
                                    skip = true;
                                } else if meta.path.is_ident("arrow_precision") {
                                    let lit: LitInt = meta.value()?.parse()?;
                                    arrow_attrs.precision = Some(lit.base10_parse()?);
                                } else if meta.path.is_ident("arrow_scale") {
                                    let lit: LitInt = meta.value()?.parse()?;
                                    arrow_attrs.scale = Some(lit.base10_parse()?);
                                } else if meta.path.is_ident("arrow_timestamp_unit") {
                                    let lit: LitStr = meta.value()?.parse()?;
                                    arrow_attrs.timestamp_unit = Some(lit.value());
                                } else if meta.path.is_ident("arrow_timezone") {
                                    let lit: LitStr = meta.value()?.parse()?;
                                    arrow_attrs.timezone = Some(lit.value());
                                } else if meta.path.is_ident("arrow_comment") {
                                    let lit: LitStr = meta.value()?.parse()?;
                                    arrow_attrs.comment = Some(lit.value());
                                } else if meta.path.is_ident("column_type") {
                                    let lit: LitStr = meta.value()?.parse()?;
                                    column_type_str = Some(lit.value());
                                } else if meta.path.is_ident("nullable") {
                                    arrow_attrs.nullable_attr = true;
                                }
                                Ok(())
                            })?;
                        }
                    }

                    if skip {
                        continue; // Skip this field
                    }

                    // Determine final nullability
                    let nullable = is_nullable || arrow_attrs.nullable_attr;

                    fields_info.push(ArrowFieldInfo {
                        name: field_name,
                        field_type: field_type.clone(),
                        column_type_str,
                        nullable,
                        arrow_attrs,
                    });
                }
            }
        }
    }

    // Generate arrow_schema() method
    let field_definitions = fields_info
        .iter()
        .map(|info| generate_field_definition(info));

    let entity_name = format_ident!("Entity");

    Ok(quote! {
        #[automatically_derived]
        impl sea_orm::ArrowSchema for #entity_name {
            fn arrow_schema() -> arrow::datatypes::Schema {
                use arrow::datatypes::{DataType, Field, Schema, TimeUnit};

                Schema::new(vec![
                    #(#field_definitions),*
                ])
            }
        }
    })
}

#[derive(Default)]
struct ArrowFieldAttrs {
    precision: Option<u8>,
    scale: Option<i8>,
    timestamp_unit: Option<String>,
    timezone: Option<String>,
    comment: Option<String>,
    nullable_attr: bool,
}

struct ArrowFieldInfo {
    name: String,
    field_type: Type,
    column_type_str: Option<String>,
    nullable: bool,
    arrow_attrs: ArrowFieldAttrs,
}

fn generate_field_definition(info: &ArrowFieldInfo) -> TokenStream {
    let field_name = &info.name;
    let nullable = info.nullable;

    // Generate DataType based on column_type or field type
    let data_type = if let Some(col_type_str) = &info.column_type_str {
        column_type_to_arrow_datatype(col_type_str, &info.arrow_attrs)
    } else {
        rust_type_to_arrow_datatype(&info.field_type, &info.arrow_attrs)
    };

    // Add metadata if comment is present
    if let Some(comment) = &info.arrow_attrs.comment {
        quote! {
            Field::new(#field_name, #data_type, #nullable)
                .with_metadata([(
                    "comment".into(),
                    #comment.into()
                )].into())
        }
    } else {
        quote! {
            Field::new(#field_name, #data_type, #nullable)
        }
    }
}

/// Map SeaORM ColumnType string to Arrow DataType
fn column_type_to_arrow_datatype(col_type: &str, arrow_attrs: &ArrowFieldAttrs) -> TokenStream {
    // Parse ColumnType variants
    if col_type.starts_with("Decimal(") {
        // Extract precision and scale from Decimal(Some((p, s)))
        let (precision, scale) = if col_type.contains("Some((") {
            // Parse "Decimal(Some((20, 4)))"
            if let Some(inner) = col_type
                .strip_prefix("Decimal(Some((")
                .and_then(|s| s.strip_suffix(")))"))
            {
                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    let p = parts[0].parse().unwrap_or(38);
                    let s = parts[1].parse().unwrap_or(10);
                    (p, s)
                } else {
                    (38, 10)
                }
            } else {
                (38, 10)
            }
        } else {
            (38, 10) // Default for Decimal(None)
        };

        // Allow arrow_precision/arrow_scale to override
        let final_precision = arrow_attrs.precision.unwrap_or(precision);
        let final_scale = arrow_attrs.scale.unwrap_or(scale);

        if final_precision <= 38 {
            quote! { DataType::Decimal128(#final_precision, #final_scale) }
        } else {
            quote! { DataType::Decimal256(#final_precision, #final_scale) }
        }
    } else if col_type.starts_with("Money(") {
        // Money type - default to Decimal128(19, 4)
        let precision = arrow_attrs.precision.unwrap_or(19);
        let scale = arrow_attrs.scale.unwrap_or(4);
        quote! { DataType::Decimal128(#precision, #scale) }
    } else if col_type == "TinyInteger" {
        quote! { DataType::Int8 }
    } else if col_type == "SmallInteger" {
        quote! { DataType::Int16 }
    } else if col_type == "Integer" {
        quote! { DataType::Int32 }
    } else if col_type == "BigInteger" {
        quote! { DataType::Int64 }
    } else if col_type == "TinyUnsigned" {
        quote! { DataType::UInt8 }
    } else if col_type == "SmallUnsigned" {
        quote! { DataType::UInt16 }
    } else if col_type == "Unsigned" {
        quote! { DataType::UInt32 }
    } else if col_type == "BigUnsigned" {
        quote! { DataType::UInt64 }
    } else if col_type == "Float" {
        quote! { DataType::Float32 }
    } else if col_type == "Double" {
        quote! { DataType::Float64 }
    } else if col_type == "Boolean" {
        quote! { DataType::Boolean }
    } else if col_type == "Text" {
        quote! { DataType::LargeUtf8 }
    } else if col_type.starts_with("String(") {
        // Parse String(StringLen::N(255)) or String(StringLen::None)
        if col_type.contains("None") || col_type.contains("Max") {
            quote! { DataType::LargeUtf8 }
        } else {
            // Try to extract length
            if let Some(inner) = col_type
                .strip_prefix("String(StringLen::N(")
                .and_then(|s| s.strip_suffix("))"))
            {
                if let Ok(n) = inner.parse::<u32>() {
                    if n <= 32767 {
                        return quote! { DataType::Utf8 };
                    }
                }
            }
            quote! { DataType::LargeUtf8 }
        }
    } else if col_type.starts_with("Char(") {
        quote! { DataType::Utf8 }
    } else if col_type == "Date" {
        quote! { DataType::Date32 }
    } else if col_type == "Time" {
        quote! { DataType::Time64(TimeUnit::Microsecond) }
    } else if col_type == "DateTime" || col_type == "Timestamp" {
        generate_timestamp_datatype(arrow_attrs, false)
    } else if col_type == "TimestampWithTimeZone" {
        generate_timestamp_datatype(arrow_attrs, true)
    } else if col_type.starts_with("Binary(") || col_type.starts_with("VarBinary(") {
        quote! { DataType::Binary }
    } else if col_type == "Json" || col_type == "JsonBinary" {
        quote! { DataType::Utf8 }
    } else if col_type == "Uuid" {
        quote! { DataType::Binary }
    } else if col_type.starts_with("Enum {") {
        quote! { DataType::Utf8 }
    } else {
        // Default fallback
        quote! { DataType::Binary }
    }
}

/// Map Rust type to Arrow DataType (when no column_type specified)
fn rust_type_to_arrow_datatype(field_type: &Type, arrow_attrs: &ArrowFieldAttrs) -> TokenStream {
    let type_string = quote! { #field_type }.to_string().replace(' ', "");

    // Strip Option<> wrapper if present
    let inner_type = if type_string.starts_with("Option<") {
        type_string
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(&type_string)
    } else {
        &type_string
    };

    match inner_type {
        "i8" => quote! { DataType::Int8 },
        "i16" => quote! { DataType::Int16 },
        "i32" => quote! { DataType::Int32 },
        "i64" => quote! { DataType::Int64 },
        "u8" => quote! { DataType::UInt8 },
        "u16" => quote! { DataType::UInt16 },
        "u32" => quote! { DataType::UInt32 },
        "u64" => quote! { DataType::UInt64 },
        "f32" => quote! { DataType::Float32 },
        "f64" => quote! { DataType::Float64 },
        "bool" => quote! { DataType::Boolean },
        "String" => quote! { DataType::Utf8 },
        s if s.contains("Decimal") => {
            let precision = arrow_attrs.precision.unwrap_or(38);
            let scale = arrow_attrs.scale.unwrap_or(10);
            if precision <= 38 {
                quote! { DataType::Decimal128(#precision, #scale) }
            } else {
                quote! { DataType::Decimal256(#precision, #scale) }
            }
        }
        s if (s.contains("DateTime") && s.contains("Offset"))
            || (s.contains("DateTime") && s.contains("Utc"))
            || (s.contains("DateTime") && s.contains("TimeZone"))
            || s.contains("Timestamp") =>
        {
            generate_timestamp_datatype(arrow_attrs, true)
        }
        s if s.contains("DateTime") => {
            generate_timestamp_datatype(arrow_attrs, arrow_attrs.timezone.is_some())
        }
        s if s.contains("Date") => quote! { DataType::Date32 },
        s if s.contains("Time") => quote! { DataType::Time64(TimeUnit::Microsecond) },
        _ => quote! { DataType::Binary }, // Safe fallback
    }
}

/// Generate timestamp DataType with optional timezone
fn generate_timestamp_datatype(arrow_attrs: &ArrowFieldAttrs, has_timezone: bool) -> TokenStream {
    let unit = match arrow_attrs.timestamp_unit.as_deref() {
        Some("Second") => quote! { TimeUnit::Second },
        Some("Millisecond") => quote! { TimeUnit::Millisecond },
        Some("Microsecond") => quote! { TimeUnit::Microsecond },
        Some("Nanosecond") => quote! { TimeUnit::Nanosecond },
        _ => quote! { TimeUnit::Microsecond }, // Default
    };

    if has_timezone {
        let tz = arrow_attrs.timezone.as_deref().unwrap_or("UTC");
        quote! { DataType::Timestamp(#unit, Some(#tz.into())) }
    } else if let Some(tz) = &arrow_attrs.timezone {
        quote! { DataType::Timestamp(#unit, Some(#tz.into())) }
    } else {
        quote! { DataType::Timestamp(#unit, None) }
    }
}
