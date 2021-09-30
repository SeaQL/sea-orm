use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse_quote;

use crate::attributes::r#type::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_child_attributes, parse_container_attributes, rename_all,
    ContainerAttributes, TypeName,
};

enum Error {
    NotSupported(String),
    Syn(syn::Error),
}

enum DataType {
    EnumStrong(syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>),
    EnumWeak(syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>),
    StructNamed(syn::punctuated::Punctuated<syn::Field, syn::token::Comma>),
    StructUnnamed(Box<syn::Field>),
}

struct DeriveType {
    attrs: ContainerAttributes,
    data_type: DataType,
    generics: syn::Generics,
    ident: syn::Ident,
}

impl DeriveType {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let attrs = parse_container_attributes(&input.attrs).map_err(Error::Syn)?;
        let (attrs, data_type) = match input.data.clone() {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                ..
            }) if unnamed.len() == 1 => {
                let field = unnamed.into_iter().next().unwrap();
                (
                    check_transparent_attributes(&input, &field).map_err(Error::Syn)?,
                    DataType::StructUnnamed(Box::new(field)),
                )
            }
            syn::Data::Enum(syn::DataEnum { variants, .. }) => match attrs.repr {
                Some(_) => (
                    check_weak_enum_attributes(&input, &variants).map_err(Error::Syn)?,
                    DataType::EnumWeak(variants),
                ),
                None => (
                    check_strong_enum_attributes(&input, &variants).map_err(Error::Syn)?,
                    DataType::EnumStrong(variants),
                ),
            },
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
                ..
            }) => (
                check_struct_attributes(&input, &named).map_err(Error::Syn)?,
                DataType::StructNamed(named),
            ),
            syn::Data::Union(_) => {
                return Err(Error::NotSupported("unions are not supported".to_string()))
            }
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(..),
                ..
            }) => {
                return Err(Error::NotSupported(
                    "structs with zero or more than one unnamed field are not supported"
                        .to_string(),
                ))
            }
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unit,
                ..
            }) => {
                return Err(Error::NotSupported(
                    "unit structs are not supported".to_string(),
                ))
            }
        };

        let ident = input.ident;
        let generics = input.generics;

        Ok(DeriveType {
            attrs,
            data_type,
            generics,
            ident,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_decode = match &self.data_type {
            DataType::EnumStrong(variants) => self.expand_enum_strong(variants),
            DataType::EnumWeak(variants) => self.expand_enum_weak(variants),
            DataType::StructNamed(fields) => self.expand_struct_named(fields),
            DataType::StructUnnamed(field) => self.expand_struct_unnamed(field),
        }?;
        let expanded_impl_query_value = self.impl_query_value()?;
        let expanded_impl_try_getable = self.impl_try_getable();

        Ok(TokenStream::from_iter([
            expanded_decode,
            expanded_impl_query_value,
            expanded_impl_try_getable,
        ]))
    }

    fn expand_enum_strong(
        &self,
        _variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self { attrs, ident, .. } = self;

        let mut tts = TokenStream::new();

        if cfg!(feature = "sqlx-mysql") {
            tts.extend(quote!(
                #[automatically_derived]
                impl ::sqlx::Type<::sqlx::MySql> for #ident {
                    fn type_info() -> ::sqlx::mysql::MySqlTypeInfo {
                        ::sqlx::mysql::MySqlTypeInfo::__enum()
                    }

                    fn compatible(ty: &::sqlx::mysql::MySqlTypeInfo) -> ::std::primitive::bool {
                        *ty == ::sqlx::mysql::MySqlTypeInfo::__enum()
                    }
                }
            ));
        }

        if cfg!(feature = "sqlx-postgres") {
            let ty_name = Self::type_name(ident, attrs.type_name.as_ref());

            tts.extend(quote!(
                #[automatically_derived]
                impl ::sqlx::Type<::sqlx::Postgres> for #ident {
                    fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                        ::sqlx::postgres::PgTypeInfo::with_name(#ty_name)
                    }
                }
            ));
        }

        if cfg!(feature = "sqlx-sqlite") {
            tts.extend(quote!(
                #[automatically_derived]
                impl sqlx::Type<::sqlx::Sqlite> for #ident {
                    fn type_info() -> ::sqlx::sqlite::SqliteTypeInfo {
                        <::std::primitive::str as ::sqlx::Type<::sqlx::Sqlite>>::type_info()
                    }

                    fn compatible(ty: &::sqlx::sqlite::SqliteTypeInfo) -> ::std::primitive::bool {
                        <&::std::primitive::str as ::sqlx::types::Type<::sqlx::sqlite::Sqlite>>::compatible(ty)
                    }
                }
            ));
        }

        Ok(tts)
    }

    fn expand_enum_weak(
        &self,
        _variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self { attrs, ident, .. } = self;

        let repr = attrs.repr.as_ref().unwrap();

        let ts = quote!(
            #[automatically_derived]
            impl<DB: ::sqlx::Database> ::sqlx::Type<DB> for #ident
            where
                #repr: ::sqlx::Type<DB>,
            {
                fn type_info() -> DB::TypeInfo {
                    <#repr as ::sqlx::Type<DB>>::type_info()
                }

                fn compatible(ty: &DB::TypeInfo) -> bool {
                    <#repr as ::sqlx::Type<DB>>::compatible(ty)
                }
            }
        );

        Ok(ts)
    }

    fn impl_query_value(&self) -> syn::Result<TokenStream> {
        let Self {
            attrs,
            data_type,
            ident,
            ..
        } = self;

        let expanded_cast_as = if attrs.cast {
            match &attrs.type_name {
                Some(type_name) => {
                    let cast_as = &type_name.val;
                    quote!(Some(#cast_as))
                }
                None => {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "cast specified but type_name was not set",
                    ))
                }
            }
        } else {
            quote!(None)
        };

        match data_type {
            DataType::EnumStrong(variants) | DataType::EnumWeak(variants) => {
                let variant_idents: Vec<_> =
                    variants.iter().map(|variant| &variant.ident).collect();
                let variant_strs: Vec<_> = variants
                    .iter()
                    .map(|variant| {
                        let variant_ident = &variant.ident;
                        let variant_attrs = parse_child_attributes(&variant.attrs).unwrap();

                        if let Some(renamed) = variant_attrs.rename {
                            quote!(#renamed)
                        } else if let Some(pattern) = attrs.rename_all {
                            let renamed = rename_all(&variant_ident.to_string(), pattern);
                            quote!(#renamed)
                        } else {
                            let name = variant_ident.to_string();
                            quote!(#name)
                        }
                    })
                    .collect();

                Ok(quote!(
                    impl ::sea_orm::sea_query::QueryValue for #ident {
                        fn query_value(&self, query_builder: &dyn QueryBuilder) -> String {
                            match self {
                                #( Self::#variant_idents => #variant_strs.to_string(), )*
                            }
                        }

                        fn primitive_value(&self) -> PrimitiveValue {
                            let self_string = match self {
                                #( Self::#variant_idents => #variant_strs.to_string(), )*
                            };
                            self_string.into()
                        }

                        fn cast_as(&self) -> Option<&'static str> {
                            #expanded_cast_as
                        }
                    }
                ))
            }
            DataType::StructUnnamed(_) => Ok(quote!(
                impl ::sea_orm::sea_query::QueryValue for #ident {
                    fn query_value(&self, query_builder: &dyn QueryBuilder) -> String {
                        self.0.query_value(query_builder)
                    }

                    fn primitive_value(&self) -> PrimitiveValue {
                        self.0.clone().into()
                    }

                    fn cast_as(&self) -> Option<&'static str> {
                        #expanded_cast_as
                    }
                }
            )),
            DataType::StructNamed(_) => todo!(),
        }
    }

    fn impl_try_getable(&self) -> TokenStream {
        let Self { ident, .. } = self;

        let mut rows = Vec::new();

        #[cfg(feature = "sqlx-mysql")]
        rows.push(quote!(
            ::sea_orm::QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<#ident>, _>(column.as_str())
                    .map_err(|e| ::sea_orm::TryGetError::DbErr(::sea_orm::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(::sea_orm::TryGetError::Null))
            }
        ));

        #[cfg(feature = "sqlx-postgres")]
        rows.push(quote!(
            ::sea_orm::QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get::<Option<#ident>, _>(column.as_str())
                    .map_err(|e| ::sea_orm::TryGetError::DbErr(::sea_orm::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(::sea_orm::TryGetError::Null))
            }
        ));

        #[cfg(feature = "sqlx-sqlite")]
        rows.push(quote!(
            ::sea_orm::QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                row.try_get::<Option<#ident>, _>(column.as_str())
                    .map_err(|e| ::sea_orm::TryGetError::DbErr(::sea_orm::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or(::sea_orm::TryGetError::Null))
            }
        ));

        quote!(
            impl ::sea_orm::TryGetable for #ident {
                fn try_get(res: &::sea_orm::QueryResult, pre: &str, col: &str) -> Result<Self, ::sea_orm::TryGetError> {
                    let column = format!("{}{}", pre, col);
                    match &res.row {
                        #(#rows)*
                    }
                }
            }
        )
    }

    fn expand_struct_named(
        &self,
        _fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self { attrs, ident, .. } = self;

        let mut tts = TokenStream::new();

        if cfg!(feature = "sqlx-postgres") {
            let ty_name = Self::type_name(ident, attrs.type_name.as_ref());

            tts.extend(quote!(
                #[automatically_derived]
                impl ::sqlx::Type<::sqlx::Postgres> for #ident {
                    fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                        ::sqlx::postgres::PgTypeInfo::with_name(#ty_name)
                    }
                }
            ));
        }

        Ok(tts)
    }

    fn expand_struct_unnamed(&self, field: &syn::Field) -> syn::Result<TokenStream> {
        let Self {
            attrs,
            generics,
            ident,
            ..
        } = self;

        let ty = &field.ty;

        let (_, ty_generics, _) = generics.split_for_impl();

        if attrs.transparent {
            let mut generics = generics.clone();
            generics
                .params
                .insert(0, parse_quote!(DB: ::sqlx::Database));
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::sqlx::Type<DB>));

            let (impl_generics, _, where_clause) = generics.split_for_impl();

            return Ok(quote!(
                #[automatically_derived]
                impl #impl_generics ::sqlx::Type< DB > for #ident #ty_generics #where_clause {
                    fn type_info() -> DB::TypeInfo {
                        <#ty as ::sqlx::Type<DB>>::type_info()
                    }

                    fn compatible(ty: &DB::TypeInfo) -> ::std::primitive::bool {
                        <#ty as ::sqlx::Type<DB>>::compatible(ty)
                    }
                }
            ));
        }

        let mut tts = TokenStream::new();

        if cfg!(feature = "sqlx-postgres") {
            let ty_name = Self::type_name(ident, attrs.type_name.as_ref());

            tts.extend(quote!(
                #[automatically_derived]
                impl ::sqlx::Type<::sqlx::postgres::Postgres> for #ident #ty_generics {
                    fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                        ::sqlx::postgres::PgTypeInfo::with_name(#ty_name)
                    }
                }
            ));
        }

        Ok(tts)
    }

    fn type_name(ident: &syn::Ident, explicit_name: Option<&TypeName>) -> TokenStream {
        explicit_name.map(|tn| tn.get()).unwrap_or_else(|| {
            let s = ident.to_string();
            quote_spanned!(ident.span()=> #s)
        })
    }
}

pub fn expand_derive_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveType::new(input) {
        Ok(model) => model.expand(),
        Err(Error::Syn(err)) => Err(err),
        Err(Error::NotSupported(msg)) => Ok(quote_spanned! {
            ident_span => compile_error!(#msg);
        }),
    }
}
