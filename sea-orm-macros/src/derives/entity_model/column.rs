use std::{borrow::Cow, iter::FromIterator};

use heck::{CamelCase, MixedCase, SnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::DeriveEntityModel;

pub struct Column<'a> {
    columns: Vec<syn::Ident>,
    entity_ident: Cow<'a, syn::Ident>,
    fields: &'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    ident: Cow<'a, syn::Ident>,
    vis: &'a syn::Visibility,
}

impl<'a> Column<'a> {
    pub fn from_entity_model(entity_model: &'a DeriveEntityModel) -> Self {
        let ident = entity_model
            .sea_attr
            .column
            .as_ref()
            .map(|column| Cow::Borrowed(column))
            .unwrap_or_else(|| Cow::Owned(format_ident!("Column")));

        let entity_ident = entity_model
            .sea_attr
            .entity
            .as_ref()
            .map(|entity| Cow::Borrowed(entity))
            .unwrap_or_else(|| Cow::Owned(format_ident!("Entity")));

        let columns = entity_model
            .fields
            .iter()
            .map(|field| {
                format_ident!(
                    "{}",
                    field.ident.as_ref().unwrap().to_string().to_camel_case()
                )
            })
            .collect();

        Column {
            columns,
            entity_ident,
            fields: &entity_model.fields,
            ident,
            vis: &entity_model.vis,
        }
    }

    pub fn expand(&self) -> TokenStream {
        let expanded_define_column = self.define_column();
        let expanded_impl_as_str = self.impl_as_str();
        let expanded_impl_column_trait = self.impl_column_trait();
        let expanded_impl_from_str = self.impl_from_str();
        let expanded_impl_iden = self.impl_iden();
        let expanded_impl_iden_static = self.impl_iden_static();

        TokenStream::from_iter([
            expanded_define_column,
            expanded_impl_as_str,
            expanded_impl_column_trait,
            expanded_impl_from_str,
            expanded_impl_iden,
            expanded_impl_iden_static,
        ])
    }

    fn define_column(&self) -> TokenStream {
        let vis = &self.vis;
        let ident = &self.ident;
        let columns = &self.columns;

        quote!(
            #[derive(Copy, Clone, Debug, sea_orm::sea_strum::EnumIter)]
            #vis enum #ident {
                #(#columns),*
            }
        )
    }

    fn impl_as_str(&self) -> TokenStream {
        let ident = &self.ident;
        let columns = &self.columns;

        let columns_as_string = self
            .fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().to_string());

        quote!(
            impl #ident {
                fn as_str(&self) -> &str {
                    match self {
                        #(Self::#columns => #columns_as_string),*
                    }
                }
            }
        )
    }

    fn impl_column_trait(&self) -> TokenStream {
        let ident = &self.ident;
        let entity_ident = &self.entity_ident;

        quote!(
            impl sea_orm::entity::ColumnTrait for #ident {
                type EntityName = #entity_ident;

                fn def(&self) -> sea_orm::entity::ColumnDef {
                    // TODO: Generate column def
                    panic!("No ColumnDef")
                }
            }
        )
    }

    fn impl_from_str(&self) -> TokenStream {
        let ident = &self.ident;

        let column_from_str_fields = self.fields.iter().map(|field| {
            let field_camel = format_ident!(
                "{}",
                field.ident.as_ref().unwrap().to_string().to_camel_case()
            );
            let column_str_snake = field_camel.to_string().to_snake_case();
            let column_str_mixed = field_camel.to_string().to_mixed_case();
            quote!(
                #column_str_snake | #column_str_mixed => Ok(#ident::#field_camel)
            )
        });

        quote!(
            impl std::str::FromStr for #ident {
                type Err = sea_orm::ColumnFromStrErr;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        #(#column_from_str_fields),*,
                        _ => Err(sea_orm::ColumnFromStrErr(format!("Failed to parse '{}' as `{}`", s, stringify!(Column)))),
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
                    write!(s, "{}", <Column as sea_orm::IdenStatic>::as_str(self)).unwrap();
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
}
