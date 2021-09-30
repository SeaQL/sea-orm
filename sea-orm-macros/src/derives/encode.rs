use proc_macro2::{Span, TokenStream};
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

struct DeriveEncode {
    attrs: ContainerAttributes,
    data_type: DataType,
    generics: syn::Generics,
    ident: syn::Ident,
}

impl DeriveEncode {
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

        Ok(DeriveEncode {
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

        let mut value_arms = Vec::new();

        for v in variants {
            let id = &v.ident;
            let variant_attrs = parse_child_attributes(&v.attrs)?;

            if let Some(rename) = variant_attrs.rename {
                value_arms.push(quote!(#ident :: #id => #rename,));
            } else if let Some(pattern) = attrs.rename_all {
                let name = rename_all(&*id.to_string(), pattern);

                value_arms.push(quote!(#ident :: #id => #name,));
            } else {
                let name = id.to_string();
                value_arms.push(quote!(#ident :: #id => #name,));
            }
        }

        Ok(quote!(
            #[automatically_derived]
            impl<'q, DB: ::sqlx::Database> ::sqlx::encode::Encode<'q, DB> for #ident
            where
                &'q ::std::primitive::str: ::sqlx::encode::Encode<'q, DB>,
            {
                fn encode_by_ref(
                    &self,
                    buf: &mut <DB as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
                ) -> ::sqlx::encode::IsNull {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&::std::primitive::str as ::sqlx::encode::Encode<'q, DB>>::encode(val, buf)
                }

                fn size_hint(&self) -> ::std::primitive::usize {
                    let val = match self {
                        #(#value_arms)*
                    };

                    <&::std::primitive::str as ::sqlx::encode::Encode<'q, DB>>::size_hint(&val)
                }
            }
        ))
    }

    fn expand_enum_weak(
        &self,
        variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    ) -> syn::Result<TokenStream> {
        let Self { attrs, ident, .. } = self;

        let repr = attrs.repr.as_ref().unwrap();

        let mut values = Vec::new();

        for v in variants {
            let id = &v.ident;
            values.push(quote!(#ident :: #id => (#ident :: #id as #repr),));
        }

        Ok(quote!(
            #[automatically_derived]
            impl<'q, DB: ::sqlx::Database> ::sqlx::encode::Encode<'q, DB> for #ident
            where
                #repr: ::sqlx::encode::Encode<'q, DB>,
            {
                fn encode_by_ref(
                    &self,
                    buf: &mut <DB as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
                ) -> ::sqlx::encode::IsNull {
                    let value = match self {
                        #(#values)*
                    };

                    <#repr as ::sqlx::encode::Encode<DB>>::encode_by_ref(&value, buf)
                }

                fn size_hint(&self) -> usize {
                    <#repr as ::sqlx::encode::Encode<DB>>::size_hint(&Default::default())
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

        if cfg!(feature = "sqlx-postgres") {
            let column_count = fields.len();

            // extract type generics
            let (_, ty_generics, _) = generics.split_for_impl();

            // add db type for impl generics & where clause
            let mut generics = generics.clone();

            let predicates = &mut generics.make_where_clause().predicates;

            for field in fields {
                let ty = &field.ty;

                predicates
                    .push(parse_quote!(#ty: for<'q> ::sqlx::encode::Encode<'q, ::sqlx::Postgres>));
                predicates.push(parse_quote!(#ty: ::sqlx::types::Type<::sqlx::Postgres>));
            }

            let (impl_generics, _, where_clause) = generics.split_for_impl();

            let writes = fields.iter().map(|field| -> syn::Stmt {
                let id = &field.ident;

                parse_quote!(
                    encoder.encode(&self. #id);
                )
            });

            let sizes = fields.iter().map(|field| -> syn::Expr {
                let id = &field.ident;
                let ty = &field.ty;

                parse_quote!(
                    <#ty as ::sqlx::encode::Encode<::sqlx::Postgres>>::size_hint(&self. #id)
                )
            });

            tts.extend(quote!(
            #[automatically_derived]
            impl #impl_generics ::sqlx::encode::Encode<'_, ::sqlx::Postgres> for #ident #ty_generics
            #where_clause
            {
                fn encode_by_ref(
                    &self,
                    buf: &mut ::sqlx::postgres::PgArgumentBuffer,
                ) -> ::sqlx::encode::IsNull {
                    let mut encoder = ::sqlx::postgres::types::PgRecordEncoder::new(buf);

                    #(#writes)*

                    encoder.finish();

                    ::sqlx::encode::IsNull::No
                }

                fn size_hint(&self) -> ::std::primitive::usize {
                    #column_count * (4 + 4) // oid (int) and length (int) for each column
                        + #(#sizes)+* // sum of the size hints for each column
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
        let lifetime = syn::Lifetime::new("'q", Span::call_site());
        let mut generics = generics.clone();
        generics
            .params
            .insert(0, syn::LifetimeDef::new(lifetime.clone()).into());

        generics
            .params
            .insert(0, parse_quote!(DB: ::sqlx::Database));
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::sqlx::encode::Encode<#lifetime, DB>));
        let (impl_generics, _, where_clause) = generics.split_for_impl();

        Ok(quote!(
            #[automatically_derived]
            impl #impl_generics ::sqlx::encode::Encode<#lifetime, DB> for #ident #ty_generics
            #where_clause
            {
                fn encode_by_ref(
                    &self,
                    buf: &mut <DB as ::sqlx::database::HasArguments<#lifetime>>::ArgumentBuffer,
                ) -> ::sqlx::encode::IsNull {
                    <#ty as ::sqlx::encode::Encode<#lifetime, DB>>::encode_by_ref(&self.0, buf)
                }

                fn produces(&self) -> Option<DB::TypeInfo> {
                    <#ty as ::sqlx::encode::Encode<#lifetime, DB>>::produces(&self.0)
                }

                fn size_hint(&self) -> usize {
                    <#ty as ::sqlx::encode::Encode<#lifetime, DB>>::size_hint(&self.0)
                }
            }
        ))
    }
}

pub fn expand_derive_encode(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveEncode::new(input) {
        Ok(model) => model.expand(),
        Err(Error::Syn(err)) => Err(err),
        Err(Error::NotSupported(msg)) => Ok(quote_spanned! {
            ident_span => compile_error!(#msg);
        }),
    }
}
