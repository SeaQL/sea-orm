use std::iter::FromIterator;

use heck::{CamelCase, MixedCase, SnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};

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

pub struct DeriveModelColumn {
    column_idents: Vec<syn::Ident>,
    entity_ident: syn::Ident,
    fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    ident: syn::Ident,
    vis: syn::Visibility,
}

impl DeriveModelColumn {
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

        let ident = sea_attr.column.unwrap_or_else(|| format_ident!("Column"));
        let entity_ident = sea_attr.entity.unwrap_or_else(|| format_ident!("Entity"));
        let column_idents = fields
            .iter()
            .map(|field| {
                format_ident!(
                    "{}",
                    field.ident.as_ref().unwrap().to_string().to_camel_case()
                )
            })
            .collect();

        Ok(DeriveModelColumn {
            column_idents,
            entity_ident,
            fields,
            ident,
            vis: input.vis,
        })
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
        let column_idents = &self.column_idents;

        quote!(
            #[derive(Copy, Clone, Debug, sea_orm::sea_strum::EnumIter)]
            #vis enum #ident {
                #(#column_idents),*
            }
        )
    }

    fn impl_as_str(&self) -> TokenStream {
        let ident = &self.ident;
        let column_idents = &self.column_idents;

        let columns_as_string = self
            .fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap().to_string());

        quote!(
            impl #ident {
                fn as_str(&self) -> &str {
                    match self {
                        #(Self::#column_idents => #columns_as_string),*
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

pub(crate) fn expand_derive_model_column(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveModelColumn::new(input) {
        Ok(model) => Ok(model.expand()),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveModelColumn on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
