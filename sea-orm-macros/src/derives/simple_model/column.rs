use heck::{CamelCase, MixedCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Field, Visibility};

pub(crate) fn expand_column(
    vis: &Visibility,
    ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let column_ident = format_ident!("{}Column", ident);
    let entity_ident = format_ident!("{}Entity", ident);

    let column_fields = fields.iter().map(|field| {
        format_ident!(
            "{}",
            field.ident.as_ref().unwrap().to_string().to_camel_case()
        )
    });
    let column_fields_cloned = column_fields.clone();
    let column_field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string());

    let column_from_str_fields = fields.iter().map(|field| {
        let field_camel = format_ident!(
            "{}",
            field.ident.as_ref().unwrap().to_string().to_camel_case()
        );
        let column_str_snake = field_camel.to_string().to_snake_case();
        let column_str_mixed = field_camel.to_string().to_mixed_case();
        quote!(
            #column_str_snake | #column_str_mixed => Ok(#column_ident::#field_camel)
        )
    });

    quote!(
        #[derive(Copy, Clone, Debug, sea_orm::sea_strum::EnumIter)]
        #vis enum #column_ident {
            #(#column_fields),*
        }

        impl #column_ident {
            fn default_as_str(&self) -> &str {
                match self {
                    #(Self::#column_fields_cloned => #column_field_names),*
                }
            }
        }

        impl sea_orm::entity::ColumnTrait for #column_ident {
            type EntityName = #entity_ident;

            fn def(&self) -> sea_orm::entity::ColumnDef {
                // TODO: Generate column def
                panic!("No ColumnDef")
            }
        }

        impl std::str::FromStr for #column_ident {
            type Err = sea_orm::ColumnFromStrErr;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#column_from_str_fields),*,
                    _ => Err(sea_orm::ColumnFromStrErr(format!("Failed to parse '{}' as `{}`", s, stringify!(#column_ident)))),
                }
            }
        }

        impl sea_orm::IdenStatic for #column_ident {
            fn as_str(&self) -> &str {
                self.default_as_str()
            }
        }

        impl sea_orm::Iden for #column_ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", <#column_ident as sea_orm::IdenStatic>::as_str(self)).unwrap();
            }
        }
    )
}
