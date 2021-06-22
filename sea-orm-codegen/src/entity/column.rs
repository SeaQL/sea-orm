use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use sea_query::{ColumnDef, ColumnSpec, ColumnType};

#[derive(Clone, Debug)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) col_type: ColumnType,
    pub(crate) auto_increment: bool,
    pub(crate) not_null: bool,
    pub(crate) unique: bool,
}

impl Column {
    pub fn get_name_snake_case(&self) -> Ident {
        format_ident!("{}", self.name.to_snake_case())
    }

    pub fn get_name_camel_case(&self) -> Ident {
        format_ident!("{}", self.name.to_camel_case())
    }

    pub fn get_rs_type(&self) -> TokenStream {
        let ident = match self.col_type {
            ColumnType::Char(_)
            | ColumnType::String(_)
            | ColumnType::Text
            | ColumnType::DateTime(_)
            | ColumnType::Timestamp(_)
            | ColumnType::Time(_)
            | ColumnType::Date
            | ColumnType::Json
            | ColumnType::JsonBinary
            | ColumnType::Custom(_) => format_ident!("String"),
            ColumnType::TinyInteger(_) => format_ident!("i8"),
            ColumnType::SmallInteger(_) => format_ident!("i16"),
            ColumnType::Integer(_) => format_ident!("i32"),
            ColumnType::BigInteger(_) => format_ident!("i64"),
            ColumnType::Float(_) | ColumnType::Decimal(_) | ColumnType::Money(_) => {
                format_ident!("f32")
            }
            ColumnType::Double(_) => format_ident!("f64"),
            ColumnType::Binary(_) => format_ident!("Vec<u8>"),
            ColumnType::Boolean => format_ident!("bool"),
        };
        match self.not_null {
            true => quote! { #ident },
            false => quote! { Option<#ident> },
        }
    }

    pub fn get_def(&self) -> TokenStream {
        let mut col_def = match &self.col_type {
            ColumnType::Char(s) => match s {
                Some(s) => quote! { ColumnType::Char(Some(#s)).def() },
                None => quote! { ColumnType::Char(None).def() },
            },
            ColumnType::String(s) => match s {
                Some(s) => quote! { ColumnType::String(Some(#s)).def() },
                None => quote! { ColumnType::String(None).def() },
            },
            ColumnType::Text => quote! { ColumnType::Text.def() },
            ColumnType::TinyInteger(s) => quote! { ColumnType::TinyInteger.def() },
            ColumnType::SmallInteger(s) => quote! { ColumnType::SmallInteger.def() },
            ColumnType::Integer(s) => quote! { ColumnType::Integer.def() },
            ColumnType::BigInteger(s) => quote! { ColumnType::BigInteger.def() },
            ColumnType::Float(s) => quote! { ColumnType::Float.def() },
            ColumnType::Double(s) => quote! { ColumnType::Double.def() },
            ColumnType::Decimal(s) => match s {
                Some((s1, s2)) => quote! { ColumnType::Decimal(Some((#s1, #s2))).def() },
                None => quote! { ColumnType::Decimal(None).def() },
            },
            ColumnType::DateTime(s) => quote! { ColumnType::DateTime.def() },
            ColumnType::Timestamp(s) => quote! { ColumnType::Timestamp.def() },
            ColumnType::Time(s) => quote! { ColumnType::Time.def() },
            ColumnType::Date => quote! { ColumnType::Date.def() },
            ColumnType::Binary(s) => quote! { ColumnType::Binary.def() },
            ColumnType::Boolean => quote! { ColumnType::Boolean.def() },
            ColumnType::Money(s) => match s {
                Some((s1, s2)) => quote! { ColumnType::Money(Some((#s1, #s2))).def() },
                None => quote! { ColumnType::Money(None).def() },
            },
            ColumnType::Json => quote! { ColumnType::Json.def() },
            ColumnType::JsonBinary => quote! { ColumnType::JsonBinary.def() },
            ColumnType::Custom(s) => {
                let s = s.to_string();
                quote! { ColumnType::Custom(#s.to_owned()).def() }
            }
        };
        if !self.not_null {
            col_def.extend(quote! {
                .null()
            });
        }
        if self.unique {
            col_def.extend(quote! {
                .unique()
            });
        }
        col_def
    }
}

impl From<&ColumnDef> for Column {
    fn from(col_def: &ColumnDef) -> Self {
        let name = col_def.get_column_name();
        let col_type = match col_def.get_column_type() {
            Some(ty) => ty.clone(),
            None => panic!("ColumnType should not be empty"),
        };
        let auto_increments: Vec<bool> = col_def
            .get_column_spec()
            .iter()
            .filter_map(|spec| match spec {
                ColumnSpec::AutoIncrement => Some(true),
                _ => None,
            })
            .collect();
        let auto_increment = !auto_increments.is_empty();
        let not_nulls: Vec<bool> = col_def
            .get_column_spec()
            .iter()
            .filter_map(|spec| match spec {
                ColumnSpec::NotNull => Some(true),
                _ => None,
            })
            .collect();
        let not_null = !not_nulls.is_empty();
        let uniques: Vec<bool> = col_def
            .get_column_spec()
            .iter()
            .filter_map(|spec| match spec {
                ColumnSpec::UniqueKey => Some(true),
                _ => None,
            })
            .collect();
        let unique = !uniques.is_empty();
        Self {
            name,
            col_type,
            auto_increment,
            not_null,
            unique,
        }
    }
}
