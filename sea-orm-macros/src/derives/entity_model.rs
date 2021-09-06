use std::iter::FromIterator;

use proc_macro2::TokenStream;
use quote::quote_spanned;

use self::{column::Column, model::Model, primary_key::PrimaryKey};

mod column;
mod model;
mod primary_key;

mod derive_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct Sea {
        pub column: Option<syn::Ident>,
        pub entity: Option<syn::Ident>,
        pub primary_key: Option<syn::Ident>,
    }
}

mod field_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct Sea {
        pub auto_increment: Option<syn::Lit>,
        pub column_type: Option<syn::Type>,
        pub primary_key: Option<()>,
    }
}

pub enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

pub struct DeriveEntityModel {
    fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    ident: syn::Ident,
    sea_attr: derive_attr::Sea,
    vis: syn::Visibility,
}

impl DeriveEntityModel {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(syn::FieldsNamed { named, .. }),
                ..
            }) => named,
            _ => return Err(Error::InputNotStruct),
        };

        let sea_attr = derive_attr::Sea::try_from_attributes(&input.attrs)
            .map_err(Error::Syn)?
            .unwrap_or_default();

        Ok(DeriveEntityModel {
            fields,
            ident: input.ident,
            sea_attr,
            vis: input.vis,
        })
    }

    pub fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_column = Column::from_entity_model(self).expand();
        let expanded_model = Model::from_entity_model(self).expand();
        let expanded_primary_key = PrimaryKey::from_entity_model(self)?.expand();

        Ok(TokenStream::from_iter([
            expanded_column,
            expanded_model,
            expanded_primary_key,
        ]))
    }
}

pub(crate) fn expand_derive_entity_model(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveEntityModel::new(input) {
        Ok(entity_model) => entity_model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive SimpleModel on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
