use heck::SnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Data, DataEnum, Fields, Variant};

pub fn expand_derive_primary_key(ident: Ident, data: Data) -> syn::Result<TokenStream> {
    let variants = match data {
        syn::Data::Enum(DataEnum { variants, .. }) => variants,
        _ => {
            return Ok(quote_spanned! {
                ident.span() => compile_error!("you can only derive DerivePrimaryKey on enums");
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

    let primary_key_value: Punctuated<_, Comma> =
        variants.iter().fold(Punctuated::new(), |mut acc, v| {
            let variant = &v.ident;
            acc.push(
                quote! { active_model.take(#ident::#variant.into_column()).unwrap().unwrap() },
            );
            acc
        });
    let mut primary_key_value = primary_key_value.to_token_stream();
    if variants.len() > 1 {
        primary_key_value = quote! {
            (#primary_key_value)
        };
    }

    Ok(quote!(
        impl sea_orm::Iden for #ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", self.as_str()).unwrap();
            }
        }

        impl sea_orm::IdenStatic for #ident {
            fn as_str(&self) -> &str {
                match self {
                    #(Self::#variant => #name),*
                }
            }
        }

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

        impl PrimaryKeyValue<Entity> for #ident {
            fn get_primary_key_value<A>(
                mut active_model: A,
            ) -> <<Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType
            where
                A: ActiveModelTrait<Entity = Entity>,
            {
                #primary_key_value
            }
        }
    ))
}
