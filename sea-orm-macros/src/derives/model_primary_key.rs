use std::iter::FromIterator;

use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};

mod derive_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct Sea {
        pub column: Option<syn::Ident>,
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

pub struct DeriveModelPrimaryKey {
    auto_increment: bool,
    column_ident: syn::Ident,
    ident: syn::Ident,
    primary_keys: Vec<syn::Ident>,
    primary_key_fields: Vec<syn::Field>,
    primary_key_type: syn::Type,
    vis: syn::Visibility,
}

impl DeriveModelPrimaryKey {
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

        let ident = sea_attr
            .primary_key
            .unwrap_or_else(|| format_ident!("PrimaryKey"));
        let column_ident = sea_attr.column.unwrap_or_else(|| format_ident!("Column"));
        let model_ident = input.ident;

        let primary_key_fields: Vec<_> = fields
            .into_iter()
            .filter(|field| {
                field_attr::Sea::try_from_attributes(&field.attrs)
                    .unwrap_or_default()
                    .unwrap_or_default()
                    .primary_key
                    .is_some()
            })
            .collect();

        let primary_keys: Vec<_> = primary_key_fields
            .iter()
            .map(|field| {
                format_ident!(
                    "{}",
                    field.ident.as_ref().unwrap().to_string().to_camel_case()
                )
            })
            .collect();

        let first_primary_key = primary_key_fields.first().ok_or_else(|| {
                Error::Syn(syn::Error::new_spanned(
                    model_ident.clone(),
                    "No primary key attribute specified. Mark your primary key(s) with #[sea(primary_key)]",
                ))
            })?;

        let primary_key_type = first_primary_key.ty.clone();

        let auto_increment = if primary_keys.len() > 1 {
            false
        } else {
            field_attr::Sea::try_from_attributes(&first_primary_key.attrs).unwrap_or_default().unwrap_or_default().auto_increment.map(|auto_increment| match auto_increment {
                syn::Lit::Bool(val) => Ok(val.value),
                _ => Err(Error::Syn(syn::Error::new_spanned(
                    model_ident.clone(),
                    "Unexpected value for auto_increment. Expected #[sea(auto_increment = true | false)]",
                ))),
            }).unwrap_or(Ok(true))?
        };

        Ok(DeriveModelPrimaryKey {
            auto_increment,
            column_ident,
            ident,
            primary_keys,
            primary_key_fields,
            primary_key_type,
            vis: input.vis,
        })
    }

    pub fn expand(&self) -> TokenStream {
        let expanded_define_primary_key = self.define_primary_key();
        let expanded_impl_as_str = self.impl_as_str();
        let expanded_impl_iden = self.impl_iden();
        let expanded_impl_iden_static = self.impl_iden_static();
        let expanded_impl_primary_key_to_column = self.impl_primary_key_to_column();
        let expanded_impl_primary_key_trait = self.impl_primary_key_trait();

        TokenStream::from_iter([
            expanded_define_primary_key,
            expanded_impl_as_str,
            expanded_impl_iden,
            expanded_impl_iden_static,
            expanded_impl_primary_key_to_column,
            expanded_impl_primary_key_trait,
        ])
    }

    fn define_primary_key(&self) -> TokenStream {
        let vis = &self.vis;
        let ident = &self.ident;
        let primary_keys = &self.primary_keys;

        quote!(
            #[derive(Copy, Clone, Debug, sea_orm::sea_strum::EnumIter)]
            #vis enum #ident {
                #(#primary_keys),*
            }
        )
    }

    fn impl_as_str(&self) -> TokenStream {
        let ident = &self.ident;
        let primary_keys = &self.primary_keys;

        let primary_keys_as_string = self
            .primary_key_fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().to_string());

        quote!(
            impl #ident {
                fn as_str(&self) -> &str {
                    match self {
                        #(Self::#primary_keys => #primary_keys_as_string),*
                    }
                }
            }
        )
    }

    fn impl_iden(&self) -> TokenStream {
        let ident = &self.ident;

        quote!(
            impl sea_orm::Iden for #ident {
                fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                    write!(s, "{}", <#ident as sea_orm::IdenStatic>::as_str(self)).unwrap();
                }
            }
        )
    }

    fn impl_iden_static(&self) -> TokenStream {
        let ident = &self.ident;

        quote!(
            impl sea_orm::IdenStatic for #ident {
                fn as_str(&self) -> &str {
                    self.as_str()
                }
            }
        )
    }

    fn impl_primary_key_to_column(&self) -> TokenStream {
        let ident = &self.ident;
        let column_ident = &self.column_ident;
        let primary_keys = &self.primary_keys;

        quote!(
            impl sea_orm::PrimaryKeyToColumn for #ident {
                type Column = #column_ident;

                fn into_column(self) -> Self::Column {
                    match self {
                        #(Self::#primary_keys => Self::Column::#primary_keys,)*
                    }
                }

                fn from_column(col: Self::Column) -> Option<Self> {
                    match col {
                        #(Self::Column::#primary_keys => Some(Self::#primary_keys),)*
                        _ => None,
                    }
                }
            }
        )
    }

    fn impl_primary_key_trait(&self) -> TokenStream {
        let ident = &self.ident;
        let primary_key_type = &self.primary_key_type;
        let auto_increment = &self.auto_increment;

        quote!(
            impl sea_orm::entity::PrimaryKeyTrait for #ident {
                type ValueType = #primary_key_type;

                fn auto_increment() -> bool {
                    #auto_increment
                }
            }
        )
    }
}

pub(crate) fn expand_derive_model_primary_key(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveModelPrimaryKey::new(input) {
        Ok(model) => Ok(model.expand()),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveModelPrimaryKey on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
