use bae::FromAttributes;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

/// Attributes to derive an ActiveModel
#[derive(Default, FromAttributes)]
pub struct SeaOrm {
    pub active_model: Option<syn::Ident>,
}

enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

struct IntoActiveModel {
    attrs: SeaOrm,
    fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    field_idents: Vec<syn::Ident>,
    ident: syn::Ident,
}

impl IntoActiveModel {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
                ..
            }) => named,
            _ => return Err(Error::InputNotStruct),
        };

        let attrs = SeaOrm::try_from_attributes(&input.attrs)
            .map_err(Error::Syn)?
            .unwrap_or_default();

        let ident = input.ident;

        let field_idents = fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().clone())
            .collect();

        Ok(IntoActiveModel {
            attrs,
            fields,
            field_idents,
            ident,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_into_active_model = self.impl_into_active_model();

        Ok(expanded_impl_into_active_model)
    }

    fn impl_into_active_model(&self) -> TokenStream {
        let Self {
            attrs,
            ident,
            field_idents,
            fields,
        } = self;

        let active_model_ident = attrs
            .active_model
            .clone()
            .unwrap_or_else(|| syn::Ident::new("ActiveModel", Span::call_site()));

        let expanded_fields_into_active_model = fields.iter().map(|field| {
            let field_ident = field.ident.as_ref().unwrap();

            quote!(
                sea_orm::IntoActiveValue::<_>::into_active_value(self.#field_ident).into()
            )
        });

        quote!(
            #[automatically_derived]
            impl sea_orm::IntoActiveModel<#active_model_ident> for #ident {
                fn into_active_model(self) -> #active_model_ident {
                    #active_model_ident {
                        #( #field_idents: #expanded_fields_into_active_model, )*
                        ..::std::default::Default::default()
                    }
                }
            }
        )
    }
}

/// Method to derive the ActiveModel from the [ActiveModelTrait](sea_orm::ActiveModelTrait)
pub fn expand_into_active_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match IntoActiveModel::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive IntoActiveModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
