use std::convert::{TryFrom, TryInto};

use heck::ToSnakeCase;
use proc_macro2::{self, TokenStream, Ident};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Attribute, DataEnum, DataStruct, DeriveInput, Fields, Variant, Error, Expr, LitStr};


fn must_be_valid_iden(name: &str) -> bool {
    // can only begin with [a-z_]
    name.chars()
        .take(1)
        .all(|c| c == '_' || c.is_ascii_alphabetic())
        && name.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn impl_iden_for_unit_struct(
    ident: &proc_macro2::Ident,
    table_name: &str,
) -> proc_macro2::TokenStream {
    let prepare = if must_be_valid_iden(table_name) {
        quote! {
            fn prepare(&self, s: &mut dyn ::std::fmt::Write, q: sea_query::Quote) {
                write!(s, "{}", q.left()).unwrap();
                self.unquoted(s);
                write!(s, "{}", q.right()).unwrap();
            }
        }
    } else {
        quote! {}
    };
    quote! {
        impl sea_query::Iden for #ident {
            #prepare

            fn unquoted(&self, s: &mut dyn ::std::fmt::Write) {
                write!(s, #table_name).unwrap();
            }
        }
    }
}

fn try_from((table_name, value): (&'a str, &'a Variant)) -> Result<Self, Self::Error> {
    let Variant {
        ident,
        fields,
        attrs,
        ..
    } = value;
    let attr = find_attr(attrs).map(IdenAttr::try_from).transpose()?;

    Self::new(ident, fields, table_name, attr)
}

fn impl_iden_for_enum<'a, T>(
    ident: &proc_macro2::Ident,
    table_name: &str,
    variants: T,
) -> proc_macro2::TokenStream
where
    T: Iterator<Item = &'a Variant>,
{
    let mut is_all_valid = true;

    let match_arms = match variants
        .map(|v| (table_name, v))
        .map(|v| {
            let v = try_from(v)?;
            is_all_valid &= v.must_be_valid_iden();
            Ok(v)
        })
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(v) => quote! { #(#v),* },
        Err(e) => return e.to_compile_error(),
    };

    let prepare = if is_all_valid {
        quote! {
            fn prepare(&self, s: &mut dyn ::std::fmt::Write, q: sea_query::Quote) {
                write!(s, "{}", q.left()).unwrap();
                self.unquoted(s);
                write!(s, "{}", q.right()).unwrap();
            }
        }
    } else {
        quote! {}
    };

    quote! {
        impl sea_query::Iden for #ident {
            #prepare

            fn unquoted(&self, s: &mut dyn ::std::fmt::Write) {
                match self {
                    #match_arms
                };
            }
        }
    }
}

pub fn expand_derive_iden(input: DeriveInput) -> syn::Result<TokenStream> {
    let DeriveInput {
        ident, data, attrs, ..
    } = input;

    let table_name = ident.to_string();

    let mut new_name: TokenStream = quote!();
    input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("sea_orm"))
        .try_for_each(|attr| {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("iden") {
                    let litstr: LitStr = meta.value()?.parse()?;
                    new_name = syn::parse_str::<TokenStream>(&litstr.value()).expect("iden attribute should contain something");
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
    let variants =
        match data {
            syn::Data::Enum(DataEnum { variants, .. }) => variants,
            syn::Data::Struct(DataStruct {
                fields: Fields::Unit,
                ..
            }) => return impl_iden_for_unit_struct(&ident, &table_name).into(),
            _ => return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveIden on unit struct or enum");
            })
        };

    if variants.is_empty() {
        return variants; //todo : find good error
    }

    let output = impl_iden_for_enum(&ident, &table_name, variants.iter());

    Ok(output)
}