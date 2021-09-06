use std::iter::FromIterator;

use heck::{CamelCase, MixedCase, SnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};

use crate::attributes::{derive_attr, field_attr};

enum Error {
    InputNotStruct,
    Syn(syn::Error),
}

struct DeriveModelColumn {
    column_idents: Vec<syn::Ident>,
    entity_ident: syn::Ident,
    fields: syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    ident: syn::Ident,
    vis: syn::Visibility,
}

impl DeriveModelColumn {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
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

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_define_column = self.define_column();
        let expanded_impl_as_str = self.impl_as_str();
        let expanded_impl_column_trait = self.impl_column_trait()?;
        let expanded_impl_from_str = self.impl_from_str();
        let expanded_impl_iden = self.impl_iden();
        let expanded_impl_iden_static = self.impl_iden_static();

        Ok(TokenStream::from_iter([
            expanded_define_column,
            expanded_impl_as_str,
            expanded_impl_column_trait,
            expanded_impl_from_str,
            expanded_impl_iden,
            expanded_impl_iden_static,
        ]))
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

    fn impl_column_trait(&self) -> syn::Result<TokenStream> {
        let Self {
            ident,
            column_idents,
            entity_ident,
            ..
        } = self;

        let field_column_defs = self
            .fields
            .iter()
            .map(|field| {
                let attr = field_attr::Sea::try_from_attributes(&field.attrs)?.unwrap_or_default();

                if let Some(column_type_raw) = attr.column_type_raw {
                    return match column_type_raw {
                        syn::Lit::Str(lit_str) => {
                            lit_str.value().parse::<TokenStream>().map_err(|_| {
                                syn::Error::new_spanned(
                                    field,
                                    "'column_type_raw' attribute not valid",
                                )
                            })
                        }
                        _ => Err(syn::Error::new_spanned(
                            field,
                            "'column_type_raw' attribute must be a string",
                        )),
                    };
                }

                let column_type = match attr.column_type {
                    Some(syn::Lit::Str(lit_str)) => {
                        let column_type = lit_str.value().parse::<TokenStream>().map_err(|_| {
                            syn::Error::new_spanned(field, "'column_type_raw' attribute not valid")
                        })?;
                        quote!(sea_orm::entity::ColumnType::#column_type.def())
                    }
                    Some(_) => {
                        return Err(syn::Error::new_spanned(
                            field,
                            "'column_type_raw' attribute must be a string",
                        ))
                    }
                    None => {
                        let column_type = Self::type_to_column_type(field.ty.clone());
                        quote!(sea_orm::entity::ColumnType::#column_type.def())
                    }
                };

                let expanded_indexed = attr.indexed.map(|_| quote!(.indexed())).unwrap_or_default();
                let expanded_null = attr.null.map(|_| quote!(.null())).unwrap_or_default();
                let expanded_unique = attr.unique.map(|_| quote!(.unique())).unwrap_or_default();

                Result::<_, syn::Error>::Ok(
                    quote!(#column_type#expanded_indexed#expanded_null#expanded_unique),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(quote!(
            impl sea_orm::entity::ColumnTrait for #ident {
                type EntityName = #entity_ident;

                fn def(&self) -> sea_orm::entity::ColumnDef {
                    match self {
                        #( Self::#column_idents => #field_column_defs, )*
                    }
                }
            }
        ))
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

    fn type_to_column_type(ty: syn::Type) -> TokenStream {
        let ty_string = ty.into_token_stream().to_string();
        match ty_string.as_str() {
            "std::string::String" | "string::String" | "String" => {
                quote!(String(None))
            }
            "char" => quote!(Char(None)),
            "i8" => quote!(TinyInteger),
            "i16" => quote!(SmallInteger),
            "i32" => quote!(Integer),
            "i64" => quote!(BigInteger),
            "f32" => quote!(Float),
            "f64" => quote!(Double),
            "Json" => quote!(Json),
            "DateTime" => quote!(DateTime),
            "DateTimeWithTimeZone" => quote!(DateTimeWithTimeZone),
            "Decimal" => quote!(Decimal(None)),
            "Uuid" => quote!(Uuid),
            "Vec < u8 >" => quote!(Binary),
            "bool" => quote!(Boolean),
            _ => {
                let ty_name = ty_string.replace(' ', "").to_snake_case();
                quote!(Custom(#ty_name.to_owned()))
            }
        }
    }
}

pub fn expand_derive_model_column(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match DeriveModelColumn::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive DeriveModelColumn on structs");
        }),
        Err(Error::Syn(err)) => Err(err),
    }
}
