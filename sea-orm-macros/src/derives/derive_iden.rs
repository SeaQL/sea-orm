use heck::ToSnakeCase;
use proc_macro2::{self, TokenStream};
use quote::{quote, quote_spanned};
use syn::{
    DataEnum, DataStruct, DeriveInput, Expr, Fields, LitStr, Variant, punctuated::Punctuated,
};

pub(super) fn is_static_iden(name: &str) -> bool {
    // can only begin with [a-z_]
    name.chars()
        .take(1)
        .all(|c| c == '_' || c.is_ascii_alphabetic())
        && name.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
}

pub(super) fn impl_iden_for_unit_struct(
    ident: &syn::Ident,
    iden_str: &str,
) -> proc_macro2::TokenStream {
    let quoted = if is_static_iden(iden_str) {
        quote! {
            fn quoted(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#iden_str)
            }
        }
    } else {
        quote! {}
    };
    quote! {
        #[automatically_derived]
        impl sea_orm::Iden for #ident {
            #quoted

            fn unquoted(&self) -> &str {
                #iden_str
            }
        }
    }
}

fn impl_iden_for_enum(
    ident: &syn::Ident,
    variants: Punctuated<Variant, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let variants = variants.iter();
    let mut all_static = true;

    let match_pair: Vec<TokenStream> = variants
        .map(|v| {
            let var_ident = &v.ident;
            let var_name = if var_ident == "Table" {
                ident
            } else {
                var_ident
            };
            let mut var_name = var_name.to_string().to_snake_case();
            v.attrs
                .iter()
                .filter(|attr| attr.path().is_ident("sea_orm"))
                .try_for_each(|attr| {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("iden") {
                            let litstr: LitStr = meta.value()?.parse()?;
                            var_name = litstr.value();
                        } else {
                            // Reads the value expression to advance the parse stream.
                            // Some parameters do not have any value,
                            // so ignoring an error occurred here.
                            let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
                        }
                        Ok(())
                    })
                })
                .expect("something something");
            all_static &= is_static_iden(&var_name);
            quote! { Self::#var_ident => #var_name }
        })
        .collect();

    let match_arms: TokenStream = quote! { #(#match_pair),* };

    let quoted = if all_static {
        quote! {
            fn quoted(&self) -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(match self {
                    #match_arms
                })
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #[automatically_derived]
        impl sea_orm::Iden for #ident {
            #quoted

            fn unquoted(&self) -> &str {
                match self {
                    #match_arms
                }
            }
        }
    }
}

pub fn expand_derive_iden(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput { ident, data, .. } = input;

    let mut new_iden: String = ident.to_string().to_snake_case();
    input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
        .try_for_each(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("iden") {
                    let litstr: LitStr = meta.value()?.parse()?;
                    new_iden = litstr.value();
                } else {
                    // Reads the value expression to advance the parse stream.
                    // Some parameters do not have any value,
                    // so ignoring an error occurred here.
                    let _: Option<Expr> = meta.value().and_then(|v| v.parse()).ok();
                }
                Ok(())
            })
        })?;

    // Currently we only support enums and unit structs
    match data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            if variants.is_empty() {
                Ok(TokenStream::new())
            } else {
                Ok(impl_iden_for_enum(&ident, variants))
            }
        }
        syn::Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => Ok(impl_iden_for_unit_struct(&ident, &new_iden)),
        _ => Ok(quote_spanned! {
            ident.span() => compile_error!("you can only derive DeriveIden on unit struct or enum");
        }),
    }
}
