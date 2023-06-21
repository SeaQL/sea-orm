use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{ext::IdentExt, Data, DataStruct, Field, Fields, Generics, Type};

use super::util::field_attr_contain_key;

struct FieldInfo {
    ident: Ident,
    flatten: bool,
    ty: Type,
}

impl From<Field> for FieldInfo {
    fn from(value: Field) -> Self {
        Self {
            flatten: field_attr_contain_key(&value, "flatten"),
            ident: format_ident!("{}", value.ident.unwrap().to_string()),
            ty: value.ty,
        }
    }
}

/// Method to derive a [QueryResult](sea_orm::QueryResult)
pub fn expand_derive_from_query_result(
    ident: Ident,
    data: Data,
    generics: Generics,
) -> syn::Result<TokenStream> {
    let arg_row = &format_ident!("row");
    let arg_pre = &format_ident!("pre");

    let fields = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => named.named,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive FromQueryResult on structs");
            })
        }
    };

    let field: Vec<FieldInfo> = fields.into_iter().map(FieldInfo::from).collect();

    let field_query: Vec<TokenStream> = field
        .iter()
        .map(|FieldInfo { ident, flatten, ty }| {
            let s = ident.unraw().to_string();
            if *flatten{
                quote! { #ident : <#ty as sea_orm::FromQueryResult>::from_query_result(#arg_row, #arg_pre)? }
            }else{
                quote! { #ident : #arg_row.try_get(#arg_pre, #s)?}
            }
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics sea_orm::FromQueryResult for #ident #ty_generics #where_clause {
            fn from_query_result(#arg_row: &sea_orm::QueryResult, #arg_pre: &str) -> std::result::Result<Self, sea_orm::DbErr> {
                Ok(Self {
                    #(#field_query),*
                })
            }
        }
    ))
}
