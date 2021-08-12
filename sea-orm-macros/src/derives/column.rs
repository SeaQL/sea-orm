use heck::SnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Data, DataEnum, Fields, Variant};

pub fn expand_derive_column(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let variants = match data {
        syn::Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DeriveColumn on enums");
            })
        }
    };

    let variant: Vec<TokenStream> = variants
        .iter()
        .map(|Variant { ident, fields, .. }| match fields {
            Fields::Named(_) => quote! { #ident{..} },
            Fields::Unnamed(_) => quote! { #ident(..) },
            Fields::Unit => quote! { #ident },
        })
        .collect();

    let name: Vec<TokenStream> = variants
        .iter()
        .map(|v| {
            let ident = v.ident.to_string().to_snake_case();
            quote! { #ident }
        })
        .collect();

    let impl_iden = expand_derive_custom_column(ident)?;

    Ok(quote!(
        #impl_iden

        impl sea_orm::IdenStatic for #ident {
            fn as_str(&self) -> &str {
                match self {
                    #(Self::#variant => #name),*
                }
            }
        }
    ))
}

pub fn expand_derive_custom_column(ident: &Ident) -> syn::Result<TokenStream> {
    Ok(quote!(
        impl sea_orm::Iden for #ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", self.as_str()).unwrap();
            }
        }
    ))
}
