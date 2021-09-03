use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Error, Field, Result, Visibility};

use crate::util::has_attribute;

pub(crate) fn expand_primary_key(
    vis: Visibility,
    ident: Ident,
    fields: Punctuated<Field, Comma>,
) -> Result<TokenStream> {
    let primary_key_ident = format_ident!("{}PrimaryKey", ident);
    let column_ident = format_ident!("{}Column", ident);

    let primary_key_fields: Vec<_> = fields
        .into_iter()
        .filter_map(|field| {
            if !has_attribute("primary_key", &field.attrs) {
                return None;
            }

            Some((
                field.ident.clone(),
                format_ident!("{}", field.ident.unwrap().to_string().to_camel_case()),
                field.ty,
            ))
        })
        .collect();

    if primary_key_fields.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "No primary key attribute specified. Mark your primary key(s) with #[primary_key]",
        ));
    }

    let primary_keys_name: Vec<_> = primary_key_fields
        .iter()
        .map(|(ident, _, _)| ident.clone().unwrap().to_string())
        .collect();
    let primary_keys: Vec<_> = primary_key_fields
        .iter()
        .map(|(_, camel_ident, _)| camel_ident)
        .collect();
    // let primary_key_type = primary_key_fields.first().unwrap().1.clone();

    let expanded = quote!(
        #[derive(Copy, Clone, Debug, sea_orm::sea_strum::EnumIter)]
        #vis enum #primary_key_ident {
            #(#primary_keys),*
        }

        impl sea_orm::entity::PrimaryKeyTrait for #primary_key_ident {
            // type ValueType = #primary_key_type;

            fn auto_increment() -> bool {
                false
            }
        }

        impl sea_orm::Iden for #primary_key_ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", <#primary_key_ident as sea_orm::IdenStatic>::as_str(self)).unwrap();
            }
        }

        impl sea_orm::IdenStatic for #primary_key_ident {
            fn as_str(&self) -> &str {
                match self {
                    #(Self::#primary_keys => #primary_keys_name),*
                }
            }
        }

        impl sea_orm::PrimaryKeyToColumn for #primary_key_ident {
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
    );

    Ok(expanded)
}
