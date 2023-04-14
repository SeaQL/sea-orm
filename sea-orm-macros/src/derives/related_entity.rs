use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use crate::derives::attributes::related_attr;

enum Error {
    InputNotEnum,
    Syn(syn::Error),
}

struct DeriveRelatedEntity {
    variants: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
}

impl DeriveRelatedEntity {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let variants = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        Ok(DeriveRelatedEntity { variants })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let variant_related_impls: Vec<TokenStream> = self
            .variants
            .iter()
            .map(|variant| {
                let attr = related_attr::SeaOrm::from_attributes(&variant.attrs)?;

                let entity = attr
                    .entity
                    .as_ref()
                    .map(Self::parse_lit_string)
                    .ok_or_else(|| {
                        syn::Error::new_spanned(variant, "Missing value for 'entity'")
                    })??;

                let to = attr
                    .to
                    .as_ref()
                    .map(Self::parse_lit_string)
                    .ok_or_else(|| syn::Error::new_spanned(variant, "Missing value for 'to'"))??;

                let via = match attr.via {
                    Some(via) => {
                        let via = Self::parse_lit_string(&via).or_else(|_| {
                            Err(syn::Error::new_spanned(variant, "Missing value for 'via'"))
                        })?;

                        quote! {
                            fn via() -> Option<RelationDef> {
                                #via
                            }
                        }
                    }
                    None => quote! {},
                };

                let to = quote! {
                    fn to() -> RelationDef {
                        #to
                    }
                };

                let inner = quote! {
                    #to

                    #via
                };

                Result::<_, syn::Error>::Ok(quote! {
                    impl Related<#entity> for Entity {
                        #inner
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(quote! {
            #(#variant_related_impls)*
        })
    }

    fn parse_lit_string(lit: &syn::Lit) -> syn::Result<TokenStream> {
        match lit {
            syn::Lit::Str(lit_str) => lit_str
                .value()
                .parse()
                .map_err(|_| syn::Error::new_spanned(lit, "attribute not valid")),
            _ => Err(syn::Error::new_spanned(lit, "attribute must be a string")),
        }
    }
}

/// Method to derive a Related enumeration
pub fn expand_derive_related_entity(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveRelatedEntity::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveRelation on enums");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
