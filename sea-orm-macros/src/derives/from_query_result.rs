use self::util::GetMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt, punctuated::Punctuated, token::Comma, Data, DataStruct, Fields, Generics, Meta,
};

struct FromQueryResultItem {
    pub skip: bool,
    pub ident: Ident,
}

impl ToTokens for FromQueryResultItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { ident, skip } = self;
        if *skip {
            tokens.extend(quote! {
                #ident: std::default::Default::default(),
            });
        } else {
            let name = ident.unraw().to_string();
            tokens.extend(quote! {
                #ident: row.try_get(pre, #name)?,
            });
        }
    }
}

struct TryFromQueryResultCheck<'a>(&'a FromQueryResultItem);

impl<'a> ToTokens for TryFromQueryResultCheck<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FromQueryResultItem { ident, skip } = self.0;
        if *skip {
            tokens.extend(quote! {
                let #ident = std::default::Default::default();
            });
        } else {
            let name = ident.unraw().to_string();
            tokens.extend(quote! {
                let #ident = match row.try_get_nullable(pre, #name) {
                    std::result::Result::Err(sea_orm::TryGetError::DbErr(err)) => {
                        return Err(err);
                    }
                    std::result::Result::Err(sea_orm::TryGetError::Null(_)) =>  std::option::Option::None,
                    std::result::Result::Ok(v) => std::option::Option::Some(v),
                };
            });
        }
    }
}

struct TryFromQueryResultAssignment<'a>(&'a FromQueryResultItem);

impl<'a> ToTokens for TryFromQueryResultAssignment<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FromQueryResultItem { ident, skip } = self.0;
        if *skip {
            tokens.extend(quote! {
                #ident,
            });
        } else {
            tokens.extend(quote! {
                #ident: match #ident {
                    std::option::Option::Some(v) => v,
                    std::option::Option::None => {
                        return std::result::Result::Ok(std::option::Option::None);
                    }
                },
            });
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
        let mut skip = false;
        for attr in parsed_field.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list.iter() {
                    skip = meta.exists("skip");
                }
            }
        }
        let ident = format_ident!("{}", parsed_field.ident.unwrap().to_string());
        fields.push(FromQueryResultItem { skip, ident });
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ident_try_init: Vec<_> = fields.iter().map(TryFromQueryResultCheck).collect();
    let ident_try_assign: Vec<_> = fields.iter().map(TryFromQueryResultAssignment).collect();

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics sea_orm::FromQueryResult for #ident #ty_generics #where_clause {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Self, sea_orm::DbErr> {
                Ok(Self {
                    #(#fields)*
                })
            }

            fn from_query_result_optional(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Option<Self>, sea_orm::DbErr> {
                #(#ident_try_init)*

                std::result::Result::Ok(std::option::Option::Some(Self {
                    #(#ident_try_assign)*
                }))
            }
        }
    ))
}
mod util {
    use syn::Meta;

    pub(super) trait GetMeta {
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
