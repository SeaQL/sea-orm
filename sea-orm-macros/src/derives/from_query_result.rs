use self::util::GetMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt, punctuated::Punctuated, token::Comma, Data, DataStruct, Fields, Generics, Meta,
};

enum ItemType {
    Normal,
    Skipped,
    Nested,
}

struct FromQueryResultItem {
    pub typ: ItemType,
    pub ident: Ident,
}

/// Initially, we try to obtain the value for each field and check if it is an ordinary DB error
/// (which we return immediatly), or a null error.
///
/// ### Background
///
/// Null errors do not necessarily mean that the deserialization as a whole fails,
/// since structs embedding the current one might have wrapped the current one in an `Option`.
/// In this case, we do not want to swallow other errors, which are very likely to actually be
/// programming errors that should be noticed (and fixed).
struct TryFromQueryResultCheck<'a>(&'a FromQueryResultItem);

impl<'a> ToTokens for TryFromQueryResultCheck<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FromQueryResultItem { ident, typ } = self.0;

        match typ {
            ItemType::Normal => {
                let name = ident.unraw().to_string();
                tokens.extend(quote! {
                    let #ident = match row.try_get_nullable(pre, #name) {
                         Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                             return Err(v);
                         }
                         v => v,
                     };
                });
            }
            ItemType::Skipped => {
                tokens.extend(quote! {
                    let #ident = std::default::Default::default();
                });
            }
            ItemType::Nested => {
                let name = ident.unraw().to_string();
                tokens.extend(quote! {
                    let #ident = match sea_orm::FromQueryResult::from_query_result_nullable(row, &format!("{pre}{}-", #name)) {
                        Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                            return Err(v);
                        }
                        v => v,
                    };
                });
            }
        }
    }
}

struct TryFromQueryResultAssignment<'a>(&'a FromQueryResultItem);

impl<'a> ToTokens for TryFromQueryResultAssignment<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FromQueryResultItem { ident, typ, .. } = self.0;

        match typ {
            ItemType::Normal | ItemType::Nested => {
                tokens.extend(quote! {
                    #ident: #ident?,
                });
            }
            ItemType::Skipped => {
                tokens.extend(quote! {
                    #ident,
                });
            }
        }
    }
}

/// Method to derive a [QueryResult](sea_orm::QueryResult)
pub fn expand_derive_from_query_result(
    ident: Ident,
    data: Data,
    generics: Generics,
) -> syn::Result<TokenStream> {
    let parsed_fields = match data {
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

    let mut fields = Vec::with_capacity(parsed_fields.len());
    for parsed_field in parsed_fields.into_iter() {
        let mut typ = ItemType::Normal;
        for attr in parsed_field.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list.iter() {
                    if meta.exists("skip") {
                        typ = ItemType::Skipped;
                    } else if meta.exists("nested") {
                        typ = ItemType::Nested;
                    }
                }
            }
        }
        let ident = format_ident!("{}", parsed_field.ident.unwrap().to_string());
        fields.push(FromQueryResultItem { typ, ident });
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ident_try_init: Vec<_> = fields.iter().map(TryFromQueryResultCheck).collect();
    let ident_try_assign: Vec<_> = fields.iter().map(TryFromQueryResultAssignment).collect();

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics sea_orm::FromQueryResult for #ident #ty_generics #where_clause {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::DbErr> {
                Ok(Self::from_query_result_nullable(row, pre)?)
            }

            fn from_query_result_nullable(row: &sea_orm::QueryResult, pre: &str) -> Result<Self, sea_orm::TryGetError> {
                #(#ident_try_init)*

                Ok(Self {
                    #(#ident_try_assign)*
                })
            }
        }
    ))
}

pub(super) mod util {
    use syn::Meta;

    pub trait GetMeta {
        fn exists(&self, k: &str) -> bool;
    }

    impl GetMeta for Meta {
        fn exists(&self, k: &str) -> bool {
            let Meta::Path(path) = self else {
                return false;
            };
            path.is_ident(k)
        }
    }
}
