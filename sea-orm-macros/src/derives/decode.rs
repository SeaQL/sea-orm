use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse_quote;

use crate::attributes::r#type::{
    check_strong_enum_attributes, check_struct_attributes, check_transparent_attributes,
    check_weak_enum_attributes, parse_child_attributes, parse_container_attributes, rename_all,
    ContainerAttributes,
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

struct DeriveDecode {
    attrs: ContainerAttributes,
    data_type: DataType,
    generics: syn::Generics,
    ident: syn::Ident,
}

impl DeriveDecode {
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

        Ok(DeriveDecode {
            attrs,
            data_type,
            generics,
            ident,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        match &self.data_type {
            DataType::EnumStrong(variants) => self.expand_enum_strong(variants),
            DataType::EnumWeak(variants) => self.expand_enum_weak(variants),
            DataType::StructNamed(fields) => self.expand_struct_named(fields),
            DataType::StructUnnamed(field) => self.expand_struct_unnamed(field),
        }
    }

    fn expand_enum_strong(
        &self,
        variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self { attrs, ident, .. } = self;

        let ident_s = ident.to_string();

        let value_arms = variants.iter().map(|v| -> syn::Arm {
            let id = &v.ident;
            let variant_attrs = parse_child_attributes(&v.attrs).unwrap();

            if let Some(rename) = variant_attrs.rename {
                parse_quote!(#rename => ::std::result::Result::Ok(#ident :: #id),)
            } else if let Some(pattern) = attrs.rename_all {
                let name = rename_all(&*id.to_string(), pattern);

                parse_quote!(#name => ::std::result::Result::Ok(#ident :: #id),)
            } else {
                let name = id.to_string();
                parse_quote!(#name => ::std::result::Result::Ok(#ident :: #id),)
            }
        });

        let values = quote! {
            match value {
                #(#value_arms)*

                _ => Err(format!("invalid value {:?} for enum {}", value, #ident_s).into())
            }
        };

        let mut tts = TokenStream::new();

        if cfg!(feature = "sqlx-mysql") {
            tts.extend(quote!(
                #[automatically_derived]
                impl<'r> ::sqlx::decode::Decode<'r, ::sqlx::mysql::MySql> for #ident {
                    fn decode(
                        value: ::sqlx::mysql::MySqlValueRef<'r>,
                    ) -> ::std::result::Result<
                        Self,
                        ::std::boxed::Box<
                            dyn ::std::error::Error
                                + 'static
                                + ::std::marker::Send
                                + ::std::marker::Sync,
                        >,
                    > {
                        let value = <&'r ::std::primitive::str as ::sqlx::decode::Decode<
                            'r,
                            ::sqlx::mysql::MySql,
                        >>::decode(value)?;

                        #values
                    }
                }
            ));
        }

        if cfg!(feature = "sqlx-postgres") {
            tts.extend(quote!(
                #[automatically_derived]
                impl<'r> ::sqlx::decode::Decode<'r, ::sqlx::postgres::Postgres> for #ident {
                    fn decode(
                        value: ::sqlx::postgres::PgValueRef<'r>,
                    ) -> ::std::result::Result<
                        Self,
                        ::std::boxed::Box<
                            dyn ::std::error::Error
                                + 'static
                                + ::std::marker::Send
                                + ::std::marker::Sync,
                        >,
                    > {
                        let value = <&'r ::std::primitive::str as ::sqlx::decode::Decode<
                            'r,
                            ::sqlx::postgres::Postgres,
                        >>::decode(value)?;

                        #values
                    }
                }
            ));
        }

        if cfg!(feature = "sqlx-sqlite") {
            tts.extend(quote!(
                #[automatically_derived]
                impl<'r> ::sqlx::decode::Decode<'r, ::sqlx::sqlite::Sqlite> for #ident {
                    fn decode(
                        value: ::sqlx::sqlite::SqliteValueRef<'r>,
                    ) -> ::std::result::Result<
                        Self,
                        ::std::boxed::Box<
                            dyn ::std::error::Error
                                + 'static
                                + ::std::marker::Send
                                + ::std::marker::Sync,
                        >,
                    > {
                        let value = <&'r ::std::primitive::str as ::sqlx::decode::Decode<
                            'r,
                            ::sqlx::sqlite::Sqlite,
                        >>::decode(value)?;

                        #values
                    }
                }
            ));
        }

        Ok(tts)
    }

    fn expand_enum_weak(
        &self,
        variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self { attrs, ident, .. } = self;

        let repr = attrs.repr.as_ref().unwrap();

        let ident_s = ident.to_string();

        let arms = variants
            .iter()
            .map(|v| {
                let id = &v.ident;
                parse_quote! {
                    _ if (#ident::#id as #repr) == value => ::std::result::Result::Ok(#ident::#id),
                }
            })
            .collect::<Vec<syn::Arm>>();

        Ok(quote!(
            #[automatically_derived]
            impl<'r, DB: ::sqlx::Database> ::sqlx::decode::Decode<'r, DB> for #ident
            where
                #repr: ::sqlx::decode::Decode<'r, DB>,
            {
                fn decode(
                    value: <DB as ::sqlx::database::HasValueRef<'r>>::ValueRef,
                ) -> ::std::result::Result<
                    Self,
                    ::std::boxed::Box<
                        dyn ::std::error::Error + 'static + ::std::marker::Send + ::std::marker::Sync,
                    >,
                > {
                    let value = <#repr as ::sqlx::decode::Decode<'r, DB>>::decode(value)?;

                    match value {
                        #(#arms)*
                        _ => ::std::result::Result::Err(::std::boxed::Box::new(::sqlx::Error::Decode(
                            ::std::format!("invalid value {:?} for enum {}", value, #ident_s).into(),
                        )))
                    }
                }
            }
        ))
    }

    fn expand_struct_named(
        &self,
        fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self {
            generics, ident, ..
        } = self;

        let mut tts = TokenStream::new();

        if cfg!(feature = "postgres") {
            // extract type generics
            let (_, ty_generics, _) = generics.split_for_impl();

            // add db type for impl generics & where clause
            let mut generics = generics.clone();
            generics.params.insert(0, parse_quote!('r));

            let predicates = &mut generics.make_where_clause().predicates;

            for field in fields {
                let ty = &field.ty;

                predicates.push(parse_quote!(#ty: ::sqlx::decode::Decode<'r, ::sqlx::Postgres>));
                predicates.push(parse_quote!(#ty: ::sqlx::types::Type<::sqlx::Postgres>));
            }

            let (impl_generics, _, where_clause) = generics.split_for_impl();

            let reads = fields.iter().map(|field| -> syn::Stmt {
                let id = &field.ident;
                let ty = &field.ty;

                parse_quote!(
                    let #id = decoder.try_decode::<#ty>()?;
                )
            });

            let names = fields.iter().map(|field| &field.ident);

            tts.extend(quote!(
            #[automatically_derived]
            impl #impl_generics ::sqlx::decode::Decode<'r, ::sqlx::Postgres> for #ident #ty_generics
            #where_clause
            {
                fn decode(
                    value: ::sqlx::postgres::PgValueRef<'r>,
                ) -> ::std::result::Result<
                    Self,
                    ::std::boxed::Box<
                        dyn ::std::error::Error
                            + 'static
                            + ::std::marker::Send
                            + ::std::marker::Sync,
                    >,
                > {
                    let mut decoder = ::sqlx::postgres::types::PgRecordDecoder::new(value)?;

                    #(#reads)*

                    ::std::result::Result::Ok(#ident {
                        #(#names),*
                    })
                }
            }
        ));
        }

        Ok(tts)
    }

    fn expand_struct_unnamed(&self, field: &syn::Field) -> syn::Result<TokenStream> {
        let Self {
            generics, ident, ..
        } = self;

        let ty = &field.ty;

        // extract type generics
        let (_, ty_generics, _) = generics.split_for_impl();

        // add db type for impl generics & where clause
        let mut generics = generics.clone();
        generics
            .params
            .insert(0, parse_quote!(DB: ::sqlx::Database));
        generics.params.insert(0, parse_quote!('r));
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::sqlx::decode::Decode<'r, DB>));
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        let tts = quote!(
            #[automatically_derived]
            impl #impl_generics ::sqlx::decode::Decode<'r, DB> for #ident #ty_generics #where_clause {
                fn decode(
                    value: <DB as ::sqlx::database::HasValueRef<'r>>::ValueRef,
                ) -> ::std::result::Result<
                    Self,
                    ::std::boxed::Box<
                        dyn ::std::error::Error + 'static + ::std::marker::Send + ::std::marker::Sync,
                    >,
                > {
                    <#ty as ::sqlx::decode::Decode<'r, DB>>::decode(value).map(Self)
                }
            }
        );

        Ok(tts)
    }
}

pub fn expand_derive_decode(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveDecode::new(input) {
        Ok(model) => model.expand(),
        Err(Error::Syn(err)) => Err(err),
        Err(Error::NotSupported(msg)) => Ok(quote_spanned! {
            ident_span => compile_error!(#msg);
        }),
    }
}
