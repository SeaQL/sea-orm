use super::attributes::value_type_attr;
use super::value_type_match::{array_type_expr, can_try_from_u64, column_type_expr};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, Type, punctuated::Punctuated, spanned::Spanned, token::Comma};

#[allow(clippy::large_enum_variant)]
enum DeriveValueType {
    TupleStruct(DeriveValueTypeStruct),
    StringLike(DeriveValueTypeString),
}

struct DeriveValueTypeStruct {
    name: syn::Ident,
    ty: Type,
    column_type: TokenStream,
    array_type: TokenStream,
    can_try_from_u64: bool,
}

#[derive(Default)]
struct DeriveValueTypeStructAttrs {
    column_type: Option<TokenStream>,
    array_type: Option<TokenStream>,
}

impl TryFrom<value_type_attr::SeaOrm> for DeriveValueTypeStructAttrs {
    type Error = syn::Error;

    fn try_from(attrs: value_type_attr::SeaOrm) -> syn::Result<Self> {
        Ok(Self {
            column_type: attrs.column_type.map(|s| s.parse()).transpose()?,
            array_type: attrs.array_type.map(|s| s.parse()).transpose()?,
        })
    }
}

struct DeriveValueTypeString {
    name: syn::Ident,
    from_str: Option<TokenStream>,
    to_str: Option<TokenStream>,
    column_type: Option<TokenStream>,
}

struct DeriveValueTypeStringAttrs {
    from_str: Option<TokenStream>,
    to_str: Option<TokenStream>,
    column_type: Option<TokenStream>,
}

impl TryFrom<value_type_attr::SeaOrm> for DeriveValueTypeStringAttrs {
    type Error = syn::Error;

    fn try_from(attrs: value_type_attr::SeaOrm) -> syn::Result<Self> {
        let value_type = attrs.value_type.map(|s| s.value());
        assert_eq!(value_type.as_deref(), Some("String"));

        Ok(Self {
            from_str: attrs.from_str.map(|s| s.parse()).transpose()?,
            to_str: attrs.to_str.map(|s| s.parse()).transpose()?,
            column_type: attrs.column_type.map(|s| s.parse()).transpose()?,
        })
    }
}

impl DeriveValueType {
    fn new(input: syn::DeriveInput) -> syn::Result<Self> {
        // Produce an error if the macro attributes are malformed
        let value_type_attr = value_type_attr::SeaOrm::try_from_attributes(&input.attrs)?;

        // If some attributes were set, inspect the optional `value_type`
        let value_type = if let Some(ref value_type_attr) = value_type_attr {
            value_type_attr.value_type.as_ref().map(|s| s.value())
        } else {
            None
        };

        // If either `value_type` is unset, or no attributes were passed, assume
        // `DeriveValueTypeStruct`. If no attrs were set, use default values.
        if value_type.is_none() || value_type_attr.is_none() {
            let value_type_attr = if let Some(value_type_attr) = value_type_attr {
                value_type_attr.try_into()?
            } else {
                DeriveValueTypeStructAttrs::default()
            };

            match input.data {
                syn::Data::Struct(syn::DataStruct {
                    fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                    ..
                }) => {
                    return DeriveValueTypeStruct::new(input.ident, value_type_attr, unnamed)
                        .map(Self::TupleStruct);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        input,
                        "You can only derive `DeriveValueType` on a struct with a single unnamed field, unless `value_type` is set.",
                    ));
                }
            }
        }

        let value_type_attr = value_type_attr.unwrap();
        let value_type = value_type.unwrap();

        match value_type.as_str() {
            "String" => DeriveValueTypeString::new(input.ident, value_type_attr.try_into()?)
                .map(Self::StringLike),
            _ => Err(syn::Error::new_spanned(
                input.ident,
                r#"Please specify value_type = "String""#,
            )),
        }
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        Ok(match self {
            Self::TupleStruct(s) => s.impl_value_type(),
            Self::StringLike(s) => s.impl_value_type(),
        })
    }
}

impl DeriveValueTypeStruct {
    fn new(
        name: Ident,
        attrs: DeriveValueTypeStructAttrs,
        fields: Punctuated<Field, Comma>,
    ) -> syn::Result<Self> {
        let Some(field) = fields.into_iter().next() else {
            return Err(syn::Error::new_spanned(
                name,
                "You can only derive `DeriveValueType` on tuple struct with 1 inner value",
            ));
        };

        let field_span = field.span();
        let ty = field.ty;
        let field_type = quote! { #ty }
            .to_string() //E.g.: "Option < String >"
            .replace(' ', ""); // Remove spaces
        let field_type = if field_type.starts_with("Option<") {
            &field_type[7..(field_type.len() - 1)] // Extract `T` out of `Option<T>`
        } else {
            field_type.as_str()
        };

        let column_type = column_type_expr(attrs.column_type, field_type, field_span);
        let array_type = array_type_expr(attrs.array_type, field_type, field_span);
        let can_try_from_u64 = can_try_from_u64(field_type);

        Ok(Self {
            name,
            ty,
            column_type,
            array_type,
            can_try_from_u64,
        })
    }

    fn impl_value_type(&self) -> TokenStream {
        let name = &self.name;
        let field_type = &self.ty;
        let column_type = &self.column_type;
        let array_type = &self.array_type;

        let try_from_u64_impl = if self.can_try_from_u64 {
            quote!(
                #[automatically_derived]
                impl sea_orm::TryFromU64 for #name {
                    fn try_from_u64(n: u64) -> Result<Self, sea_orm::DbErr> {
                        use std::convert::TryInto;
                        Ok(Self(n.try_into().map_err(|e| sea_orm::DbErr::TryIntoErr {
                            from: stringify!(u64),
                            into: stringify!(#name),
                            source: std::sync::Arc::new(e),
                        })?))
                    }
                }
            )
        } else {
            quote!()
        };

        let impl_not_u8 = if cfg!(feature = "postgres-array") {
            quote!(
                #[automatically_derived]
                impl sea_orm::sea_query::value::with_array::NotU8 for #ident {}
            )
        } else {
            quote!()
        };

        quote!(
            #[automatically_derived]
            impl std::convert::From<#name> for sea_orm::Value {
                fn from(source: #name) -> Self {
                    source.0.into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #name {
                fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I)
                    -> std::result::Result<Self, sea_orm::TryGetError> {
                    <#field_type as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| #name(v))
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::ValueType for #name {
                fn try_from(v: sea_orm::Value) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    <#field_type as sea_orm::sea_query::ValueType>::try_from(v).map(|v| #name(v))
                }

                fn type_name() -> std::string::String {
                    stringify!(#name).to_owned()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    #array_type
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    #column_type
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::Nullable for #name {
                fn null() -> sea_orm::Value {
                    <#field_type as sea_orm::sea_query::Nullable>::null()
                }
            }

            #[automatically_derived]
            impl sea_orm::IntoActiveValue<#name> for #name {
                fn into_active_value(self) -> sea_orm::ActiveValue<#name> {
                    sea_orm::ActiveValue::Set(self)
                }
            }

            #try_from_u64_impl

            #impl_not_u8
        )
    }
}

impl DeriveValueTypeString {
    fn new(name: Ident, attrs: DeriveValueTypeStringAttrs) -> syn::Result<Self> {
        Ok(Self {
            name,
            from_str: attrs.from_str,
            to_str: attrs.to_str,
            column_type: attrs.column_type,
        })
    }

    fn impl_value_type(&self) -> TokenStream {
        let name = &self.name;
        let from_str = match &self.from_str {
            Some(from_str) => from_str,
            None => &quote!(std::str::FromStr::from_str),
        };
        let to_str = match &self.to_str {
            Some(to_str) => to_str,
            None => &quote!(std::string::ToString::to_string),
        };
        let column_type = match &self.column_type {
            Some(column_type) => column_type,
            None => &quote!(String(sea_orm::sea_query::StringLen::None)),
        };

        let impl_not_u8 = if cfg!(feature = "postgres-array") {
            quote!(
                #[automatically_derived]
                impl sea_orm::sea_query::value::with_array::NotU8 for #ident {}
            )
        } else {
            quote!()
        };

        quote!(
            #[automatically_derived]
            impl std::convert::From<#name> for sea_orm::Value {
                fn from(source: #name) -> Self {
                    #to_str(&source).into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #name {
                fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I)
                    -> std::result::Result<Self, sea_orm::TryGetError> {
                    let string = String::try_get_by(res, idx)?;
                    #from_str(&string).map_err(|err| {
                        sea_orm::TryGetError::DbErr(sea_orm::DbErr::TryIntoErr {
                            from: "String",
                            into: stringify!(#name),
                            source: std::sync::Arc::new(err),
                        })
                    })
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::ValueType for #name {
                fn try_from(v: sea_orm::Value) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    let string = <String as sea_orm::sea_query::ValueType>::try_from(v)?;
                    #from_str(&string).map_err(|_| sea_orm::sea_query::ValueTypeErr)
                }

                fn type_name() -> std::string::String {
                    stringify!(#name).to_owned()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    sea_orm::sea_query::ArrayType::String
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    sea_orm::sea_query::ColumnType::#column_type
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::Nullable for #name {
                fn null() -> sea_orm::Value {
                    sea_orm::Value::String(None)
                }
            }

            #[automatically_derived]
            impl sea_orm::IntoActiveValue<#name> for #name {
                fn into_active_value(self) -> sea_orm::ActiveValue<#name> {
                    sea_orm::ActiveValue::Set(self)
                }
            }

            #impl_not_u8
        )
    }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    DeriveValueType::new(input)?.expand()
}
