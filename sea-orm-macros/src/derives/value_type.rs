// pub use sea_query::ArrayType;
// use heck::ToUpperCamelCase;
// use proc_macro2::Span;
use proc_macro2::TokenStream;
// use quote::format_ident;
use quote::{quote, ToTokens};
// use quote::quote_spanned;
// use syn::punctuated::Punctuated;
// use syn::spanned::Spanned;
use syn::Token;
// use syn::Expr;
// use syn::Meta;
use syn::Data;

// use self::util::GetAsKVMeta;
// use crate::{DeriveEntityModel, EnumIter, DeriveRelation};

struct DeriveValueType{
    name: syn::Ident,
    internal: Data,
    // tablename: String,
}

impl DeriveValueType {
    pub fn new(input: syn::DeriveInput) -> Result<Self, syn::Error> {
        let internal = input.data;
        let name = input.ident;
        // let tablename = "ab".to_string(); //todo

        Ok(
            DeriveValueType { name, internal }
        )
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_entity_name = self.impl_entity_name();

        Ok(expanded_impl_entity_name)
    }

    fn impl_entity_name(&self) -> TokenStream {
        let name = &self.name;
        let syn::Data::Struct(ty) = &self.internal else {panic!()};
        let internal_type = ty.struct_token;
        // let tablename = &self.tablename;

        quote!(
            use sea_orm::entity::prelude::*;

            #[doc = " Generated by sea-orm-macros"]
            #[derive(Clone, Debug, PartialEq, Eq)]
            pub struct #name(pub #internal_type);

            #[automatically_derived]
            impl From<#name> for Value {
                fn from(source: #name) -> Self {
                    source.0.into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #name {
                fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
                    <#internal_type as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| #name(v))
                }
            }

            #[automatically_derived]
            impl sea_query::ValueType for #name {
                fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
                    <#internal_type as sea_query::ValueType>::try_from(v).map(|v| #name(v))
                }

                fn type_name() -> String {
                    stringify!(#name).to_owned()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    <#internal_type as sea_orm::sea_query::ValueType>::array_type()
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    <#internal_type as sea_orm::sea_query::ValueType>::column_type()
                }
            }
        )

    }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    // let ident_span = input.ident.span();

    match DeriveValueType::new(input) {
        Ok(value_type) => value_type.expand(),
        Err(err) => Err(err),
    }
}