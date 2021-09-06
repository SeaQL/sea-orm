use std::{borrow::Cow, iter::FromIterator};

use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{field_attr, DeriveEntityModel};

pub struct PrimaryKey<'a> {
    auto_increment: bool,
    column_ident: Cow<'a, syn::Ident>,
    ident: Cow<'a, syn::Ident>,
    primary_keys: Vec<syn::Ident>,
    primary_key_fields: Vec<&'a syn::Field>,
    primary_key_type: &'a syn::Type,
    vis: &'a syn::Visibility,
}

impl<'a> PrimaryKey<'a> {
    pub fn from_entity_model(entity_model: &'a DeriveEntityModel) -> syn::Result<Self> {
        let ident = entity_model
            .sea_attr
            .primary_key
            .as_ref()
            .map(|primary_key| Cow::Borrowed(primary_key))
            .unwrap_or_else(|| Cow::Owned(format_ident!("PrimaryKey")));

        let column_ident = entity_model
            .sea_attr
            .column
            .as_ref()
            .map(|column| Cow::Borrowed(column))
            .unwrap_or_else(|| Cow::Owned(format_ident!("Column")));

        let primary_key_fields: Vec<_> = entity_model
            .fields
            .iter()
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
                syn::Error::new_spanned(
                    entity_model.ident.clone(),
                    "No primary key attribute specified. Mark your primary key(s) with #[sea(primary_key)]",
                )
            })?;

        let primary_key_type = &first_primary_key.ty;

        let auto_increment = if primary_keys.len() > 1 {
            false
        } else {
            field_attr::Sea::try_from_attributes(&first_primary_key.attrs).unwrap_or_default().unwrap_or_default().auto_increment.map(|auto_increment| match auto_increment {
                syn::Lit::Bool(val) => Ok(val.value),
                _ => Err(syn::Error::new_spanned(
                    entity_model.ident.clone(),
                    "Unexpected value for auto_increment. Expected #[sea(auto_increment = true | false)]",
                )),
            }).unwrap_or(Ok(true))?
        };

        Ok(PrimaryKey {
            auto_increment,
            column_ident,
            ident,
            primary_keys,
            primary_key_fields,
            primary_key_type,
            vis: &entity_model.vis,
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
