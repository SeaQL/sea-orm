// pub use sea_query::ArrayType;
// use sea_orm::entity::prelude::*;
// use heck::ToUpperCamelCase;
// use proc_macro2::Span;
use proc_macro2::TokenStream;
// use quote::format_ident;
use quote::quote;
// use quote::quote_spanned;
// use syn::punctuated::Punctuated;
// use syn::spanned::Spanned;
// use syn::token::Comma;
// use syn::Expr;

// use syn::Meta;

// use self::util::GetAsKVMeta;
// use crate::{DeriveEntityModel, EnumIter, DeriveRelation};

// #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
// #[sea_orm(table_name = "json_vec")]
// pub struct Model {
//     #[sea_orm(primary_key)]
//     pub id: i32,
//     pub str_vec: StringVec,
// }

// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
// pub enum Relation {}

// impl ActiveModelBehavior for ActiveModel {}

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct StringVec(pub Vec<String>);

// impl From<StringVec> for Value {
//     fn from(source: StringVec) -> Self {
//         source.0.into()
//     }
// }

// impl sea_orm::TryGetable for StringVec {
//     fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
//         <Vec<String> as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| StringVec(v))
//     }
// }

struct DeriveValueType{
    value: String,
}

// impl sea_query::ValueType for DeriveValueType {
//     fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
//         <Vec<String> as sea_query::ValueType>::try_from(v).map(|v| StringVec(v))
//     }

//     fn type_name() -> String {
//         stringify!(StringVec).to_owned()
//     }

//     fn array_type() -> ::ArrayType {
//         <Vec<String> as ValueType>::array_type()
//     }

//     fn column_type() -> ColumnType {
//         <Vec<String> as ValueType>::column_type()
//     }
// }

impl DeriveValueType {
    pub fn new(input: syn::DeriveInput) -> Result<Self, syn::Error> {
        let value = input.data;

        Ok(
            DeriveValueType { value:  }
        )
    }

    let expanded = quote! {
        impl sea_query::ValueType for DeriveValueType {
            fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
                <Vec<String> as sea_query::ValueType>::try_from(v).map(|v| StringVec(v))
            }
        
            fn type_name() -> String {
                stringify!(StringVec).to_owned()
            }
        
            fn array_type() -> ArrayType {
                <Vec<String> as ValueType>::array_type()
            }
        
            fn column_type() -> ColumnType {
                <Vec<String> as ValueType>::column_type()
            }
        }
    };
    expanded

    fn expand(&self) -> TokenStream {
        quote!()
    }

//     fn impl_entity_name(&self) -> TokenStream {
//         let ident = &self.ident;

//         quote!(
//             #[automatically_derived]
//             impl sea_orm::entity::EntityName for #ident {

//             }
//         )
//     }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveValueType::new(input) {
        Ok(value_type) => Ok(value_type.expand()),
        Err(err) => Err(err),
    }
}