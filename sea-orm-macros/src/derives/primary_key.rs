use super::impl_iden;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Data, DataEnum, Fields};

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

    let mut into_column_arms = Vec::new();
    let mut from_column_arms = Vec::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        let ident_str = variant_ident.to_string();
        let is_fake_pk = ident_str == "FakePrimaryKey";

        let field_pattern = match &variant.fields {
            Fields::Named(_) => quote! { #variant_ident{..} },
            Fields::Unnamed(_) => quote! { #variant_ident(..) },
            Fields::Unit => quote! { #variant_ident },
        };

        if is_fake_pk {
            // FakePrimaryKey is intentionally not added to columns_enum/all_columns, as it should not
            // be exposed for querying, and exists only as a primary key for the trait
            // when a relation has no primary keys defined
        } else {
            into_column_arms.push(quote! {
                Self::#field_pattern => Self::Column::#variant_ident
            });
            from_column_arms.push(quote! {
                Self::Column::#variant_ident => Some(Self::#variant_ident)
            });
        }
    }

    Ok(quote!(
        #[automatically_derived]
        impl sea_orm::PrimaryKeyToColumn for #ident {
            type Column = Column;

            fn into_column(self) -> Self::Column {
                match self {
                    #(#into_column_arms,)*
                    _ => panic!("FakePrimaryKey cannot be converted to a Column as it is a shadow primary key"),
                }
            }

            fn from_column(col: Self::Column) -> Option<Self> {
                match col {
                    #(#from_column_arms,)*
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
