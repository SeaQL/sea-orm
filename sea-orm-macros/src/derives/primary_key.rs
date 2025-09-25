use super::impl_iden;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Data, DataEnum, Fields, Variant};

fn impl_primary_key_to_column(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let variants = match data {
        syn::Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DerivePrimaryKey on enums");
            });
        }
    };

    if variants.is_empty() {
        return Ok(quote_spanned! {
            ident.span() => compile_error!("Entity must have a primary key column. See <https://github.com/SeaQL/sea-orm/issues/485> for details.");
        });
    }

    let variant: Vec<TokenStream> = variants
        .iter()
        .map(|Variant { ident, fields, .. }| match fields {
            Fields::Named(_) => quote! { #ident{..} },
            Fields::Unnamed(_) => quote! { #ident(..) },
            Fields::Unit => quote! { #ident },
        })
        .collect();

    Ok(quote!(
        #[automatically_derived]
        impl sea_orm::PrimaryKeyToColumn for #ident {
            type Column = Column;

            fn into_column(self) -> Self::Column {
                match self {
                    #(Self::#variant => Self::Column::#variant,)*
                }
            }

            fn from_column(col: Self::Column) -> Option<Self> {
                match col {
                    #(Self::Column::#variant => Some(Self::#variant),)*
                    _ => None,
                }
            }
        }
    ))
}

/// Method to derive a Primary Key for a Model using the [PrimaryKeyTrait](sea_orm::PrimaryKeyTrait)
pub fn expand_derive_primary_key(ident: &Ident, data: &Data) -> syn::Result<TokenStream> {
    let impl_primary_key_to_column = impl_primary_key_to_column(ident, data)?;
    let impl_iden = impl_iden(ident, data)?;

    Ok(quote!(
        #impl_primary_key_to_column

        #impl_iden
    ))
}
