use super::attributes::value_type_attr;
use super::value_type_match::{array_type_expr, can_try_from_u64, column_type_expr};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Type, spanned::Spanned};

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

struct DeriveValueTypeString {
    name: syn::Ident,
    from_str: Option<TokenStream>,
    to_str: Option<TokenStream>,
}

impl DeriveValueType {
    fn new(input: syn::DeriveInput) -> syn::Result<Self> {
        match &input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(_),
                ..
            }) => DeriveValueTypeStruct::new(input).map(Self::TupleStruct),
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(_),
                ..
            })
            | syn::Data::Enum(DataEnum { .. }) => {
                DeriveValueTypeString::new(input).map(Self::StringLike)
            }
            _ => Err(syn::Error::new_spanned(
                input,
                "You can only derive `DeriveValueType` on struct or enum",
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
    fn new(input: syn::DeriveInput) -> syn::Result<Self> {
        let name = input.ident;
        let fields = match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                ..
            }) => unnamed,
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "You can only derive `DeriveValueType` on struct",
                ));
            }
        };

        let Some(field) = fields.into_iter().next() else {
            return Err(syn::Error::new_spanned(
                name,
                "You can only derive `DeriveValueType` on tuple struct with 1 inner value",
            ));
        };

        let mut column_type = None;
        let mut array_type = None;

        if let Ok(value_type_attr) = value_type_attr::SeaOrm::from_attributes(&input.attrs) {
            column_type = value_type_attr.column_type.map(|s| s.parse()).transpose()?;
            array_type = value_type_attr.array_type.map(|s| s.parse()).transpose()?;
        }

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

        let column_type = column_type_expr(column_type, field_type, field_span);
        let array_type = array_type_expr(array_type, field_type, field_span);
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

            #try_from_u64_impl
        )
    }
}

impl DeriveValueTypeString {
    fn new(input: syn::DeriveInput) -> syn::Result<Self> {
        let name = input.ident;
        let mut from_str = None;
        let mut to_str = None;
        let mut value_type = None;

        if let Ok(value_type_attr) = value_type_attr::SeaOrm::from_attributes(&input.attrs) {
            from_str = value_type_attr.from_str.map(|s| s.parse()).transpose()?;
            to_str = value_type_attr.to_str.map(|s| s.parse()).transpose()?;
            value_type = value_type_attr.value_type.map(|s| s.value());
        }

        match value_type.as_deref() {
            Some("String") => (),
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    r#"Please specify value_type = "String""#,
                ));
            }
        }

        Ok(Self {
            name,
            from_str,
            to_str,
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
                    sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None)
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::Nullable for #name {
                fn null() -> sea_orm::Value {
                    sea_orm::Value::String(None)
                }
            }
        )
    }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    DeriveValueType::new(input)?.expand()
}
