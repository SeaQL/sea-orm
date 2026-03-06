use std::collections::{HashMap, hash_map::Entry};

use super::util::GetMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Data, DataStruct, DeriveInput, Error, Fields, Generics, Meta, ext::IdentExt,
    punctuated::Punctuated, token::Comma,
};

pub(super) enum ItemType {
    Flat,
    Skip,
    Nested { prefix: Option<String> },
}

pub(super) struct DeriveFromQueryResult {
    pub ident: syn::Ident,
    pub generics: Generics,
    pub fields: Vec<FromQueryResultItem>,
}

pub(super) struct FromQueryResultItem {
    pub typ: ItemType,
    pub ident: Ident,
    pub alias: Option<String>,
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
struct TryFromQueryResultCheck<'a>(bool, &'a FromQueryResultItem);

impl ToTokens for TryFromQueryResultCheck<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FromQueryResultItem { ident, typ, alias } = self.1;

        match typ {
            ItemType::Flat => {
                let name = alias
                    .to_owned()
                    .unwrap_or_else(|| ident.unraw().to_string());
                tokens.extend(quote! {
                    let #ident = match row.try_get_nullable(pre, #name) {
                        Err(v @ sea_orm::TryGetError::DbErr(_)) => {
                            return Err(v);
                        }
                        v => v,
                    };
                });
            }
            ItemType::Skip => {
                tokens.extend(quote! {
                    let #ident = std::default::Default::default();
                });
            }
            ItemType::Nested { prefix } => {
                let prefix = match (self.0, prefix) {
                    (_, Some(p)) => quote! { &format!("{pre}{}", #p) },
                    (true, None) => {
                        let name = ident.unraw().to_string();
                        quote! { &format!("{pre}{}_", #name) }
                    }
                    (false, None) => quote! { pre },
                };

                tokens.extend(quote! {
                    let #ident = match sea_orm::FromQueryResult::from_query_result_nullable(row, #prefix) {
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

impl ToTokens for TryFromQueryResultAssignment<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FromQueryResultItem { ident, typ, .. } = self.0;

        match typ {
            ItemType::Flat | ItemType::Nested { .. } => {
                tokens.extend(quote! {
                    #ident: #ident?,
                });
            }
            ItemType::Skip => {
                tokens.extend(quote! {
                    #ident,
                });
            }
        }
    }
}

impl DeriveFromQueryResult {
    fn new(
        DeriveInput {
            ident,
            data,
            generics,
            ..
        }: DeriveInput,
    ) -> Result<Self, Error> {
        let parsed_fields = match data {
            Data::Struct(DataStruct {
                fields: Fields::Named(named),
                ..
            }) => named.named,
            _ => {
                return Err(Error::new(
                    ident.span(),
                    "you can only derive `FromQueryResult` on named struct",
                ));
            }
        };

        let mut fields = Vec::with_capacity(parsed_fields.len());
        let mut seen_nested: HashMap<(syn::Type, Option<String>), TokenStream> = HashMap::new();

        for parsed_field in parsed_fields {
            let mut typ = ItemType::Flat;
            let mut alias = None;
            for attr in parsed_field.attrs.iter() {
                if !attr.path().is_ident("sea_orm") {
                    continue;
                }
                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                {
                    for meta in list.iter() {
                        if meta.exists("skip") {
                            typ = ItemType::Skip;
                        } else if meta.exists("nested") {
                            typ = ItemType::Nested { prefix: None };
                        } else if let Some(list) = meta.get_list_args("nested") {
                            let mut prefix = None;

                            for m in list.iter() {
                                match m.get_as_kv("prefix") {
                                    Some(p) => prefix = Some(p),
                                    None => {
                                        return Err(Error::new_spanned(
                                            m,
                                            "invalid nested attribute, expected `prefix = \"...\"`",
                                        ));
                                    }
                                }
                            }

                            typ = ItemType::Nested { prefix };
                        } else {
                            alias = meta
                                .get_as_kv("from_alias")
                                .or_else(|| meta.get_as_kv("alias"));
                        }
                    }
                }
            }

            let field_tokens = parsed_field.to_token_stream();
            let ident = parsed_field.ident.unwrap();

            if let ItemType::Nested { ref prefix } = typ {
                let key = (parsed_field.ty, prefix.clone());
                match seen_nested.entry(key) {
                    Entry::Occupied(e) => {
                        let msg = match prefix {
                            Some(p) => format!(
                                "multiple nested fields with the same type share prefix \"{p}\""
                            ),
                            None => {
                                "multiple nested fields with the same type must have a `prefix`: \
                                   use `#[sea_orm(nested(prefix = \"...\"))]`"
                                    .to_string()
                            }
                        };
                        let mut err = Error::new_spanned(&field_tokens, msg);
                        err.combine(Error::new_spanned(e.get(), "first defined here"));
                        return Err(err);
                    }
                    Entry::Vacant(e) => {
                        e.insert(field_tokens);
                    }
                }
            }

            fields.push(FromQueryResultItem { typ, ident, alias });
        }

        Ok(Self {
            ident,
            generics,
            fields,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        Ok(self.impl_from_query_result(false))
    }

    pub(super) fn impl_from_query_result(&self, prefix: bool) -> TokenStream {
        let Self {
            ident,
            generics,
            fields,
        } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let ident_try_init: Vec<_> = fields
            .iter()
            .map(|s| TryFromQueryResultCheck(prefix, s))
            .collect();
        let ident_try_assign: Vec<_> = fields.iter().map(TryFromQueryResultAssignment).collect();

        quote!(
            #[automatically_derived]
            impl #impl_generics sea_orm::FromQueryResult for #ident #ty_generics #where_clause {
                fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Self, sea_orm::DbErr> {
                    Ok(Self::from_query_result_nullable(row, pre)?)
                }

                fn from_query_result_nullable(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Self, sea_orm::TryGetError> {
                    #(#ident_try_init)*

                    Ok(Self {
                        #(#ident_try_assign)*
                    })
                }
            }
        )
    }
}

pub fn expand_derive_from_query_result(input: DeriveInput) -> syn::Result<TokenStream> {
    DeriveFromQueryResult::new(input)?.expand()
}
