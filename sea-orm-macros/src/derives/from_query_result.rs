use self::util::GetMeta;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    ext::IdentExt, punctuated::Punctuated, token::Comma, Data, DataStruct, Fields, Generics, Meta,
};

pub struct FromQueryResultItem {
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

/// Method to derive a [QueryResult](sea_orm::QueryResult)
pub fn expand_derive_from_query_result(
    ident: Ident,
    data: Data,
    generics: Generics,
) -> syn::Result<TokenStream> {
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
    let mut field = Vec::with_capacity(fields.len());

    for parsed_field in fields.into_iter() {
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
        field.push(FromQueryResultItem { skip, ident });
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote!(
        #[automatically_derived]
        impl #impl_generics sea_orm::FromQueryResult for #ident #ty_generics #where_clause {
            fn from_query_result(row: &sea_orm::QueryResult, pre: &str) -> std::result::Result<Self, sea_orm::DbErr> {
                Ok(Self {
                    #(#field)*
                })
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
