use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};

mod derive_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct Sea {
        pub entity: Option<syn::Ident>,
    }
}

mod field_attr {
    use bae::FromAttributes;

    #[derive(FromAttributes)]
    pub struct Sea {
        pub belongs_to: syn::Ident,
        pub from: syn::Ident,
        pub to: syn::Ident,
    }
}

pub enum Error {
    InputNotEnum,
    Syn(syn::Error),
}

pub struct DeriveRelation {
    entity_ident: syn::Ident,
    ident: syn::Ident,
    variants: syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
}

impl DeriveRelation {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let variants = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        let sea_attr = derive_attr::Sea::try_from_attributes(&input.attrs)
            .map_err(Error::Syn)?
            .unwrap_or_default();

        let ident = input.ident;
        let entity_ident = sea_attr.entity.unwrap_or_else(|| format_ident!("Entity"));

        Ok(DeriveRelation {
            entity_ident,
            ident,
            variants,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_relation_trait = self.impl_relation_trait()?;

        Ok(expanded_impl_relation_trait)
    }

    fn impl_relation_trait(&self) -> syn::Result<TokenStream> {
        let ident = &self.ident;
        let entity_ident = &self.entity_ident;
        let no_relation_def_msg = format!("No RelationDef for {}", ident);

        let variant_relation_defs: Vec<TokenStream> = self
            .variants
            .iter()
            .map(|variant| {
                let variant_ident = &variant.ident;
                let field_attr::Sea {
                    belongs_to,
                    from,
                    to,
                } = field_attr::Sea::from_attributes(&variant.attrs)?;

                Result::<_, syn::Error>::Ok(quote!(
                    Self::#variant_ident => #entity_ident::belongs_to(#belongs_to)
                        .from(#from)
                        .to(#to)
                        .into()
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(quote!(
            impl sea_orm::entity::RelationTrait for #ident {
                fn def(&self) -> sea_orm::entity::RelationDef {
                    match self {
                        #( #variant_relation_defs, )*
                        _ => panic!(#no_relation_def_msg)
                    }
                }
            }
        ))
    }
}

pub(crate) fn expand_derive_relation(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveRelation::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveRelation on enums");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
