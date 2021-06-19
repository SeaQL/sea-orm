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
            ColumnType::Float(_)
            | ColumnType::Decimal(_)
            | ColumnType::Money(_) => format_ident!("f32"),
            ColumnType::Double(_) => format_ident!("f64"),
            ColumnType::Binary(_) => format_ident!("Vec<u8>"),
            ColumnType::Boolean => format_ident!("bool"),
        };
        match self.not_null {
            true => quote! { #ident },
            false => quote! { Option<#ident> },
        }
    }

    pub fn get_type(&self) -> TokenStream {
        match &self.col_type {
            ColumnType::Char(s) => match s {
                Some(s) => quote! { ColumnType::Char(Some(#s)) },
                None => quote! { ColumnType::Char(None) },
            },
            ColumnType::String(s) => match s {
                Some(s) => quote! { ColumnType::String(Some(#s)) },
                None => quote! { ColumnType::String(None) },
            },
            ColumnType::Text => quote! { ColumnType::Text },
            ColumnType::TinyInteger(s) => match s {
                Some(s) => quote! { ColumnType::TinyInteger(Some(#s)) },
                None => quote! { ColumnType::TinyInteger(None) },
            },
            ColumnType::SmallInteger(s) => match s {
                Some(s) => quote! { ColumnType::SmallInteger(Some(#s)) },
                None => quote! { ColumnType::SmallInteger(None) },
            },
            ColumnType::Integer(s) => match s {
                Some(s) => quote! { ColumnType::Integer(Some(#s)) },
                None => quote! { ColumnType::Integer(None) },
            },
            ColumnType::BigInteger(s) => match s {
                Some(s) => quote! { ColumnType::BigInteger(Some(#s)) },
                None => quote! { ColumnType::BigInteger(None) },
            },
            ColumnType::Float(s) => match s {
                Some(s) => quote! { ColumnType::Float(Some(#s)) },
                None => quote! { ColumnType::Float(None) },
            },
            ColumnType::Double(s) => match s {
                Some(s) => quote! { ColumnType::Double(Some(#s)) },
                None => quote! { ColumnType::Double(None) },
            },
            ColumnType::Decimal(s) => match s {
                Some((s1, s2)) => quote! { ColumnType::Decimal(Some((#s1, #s2))) },
                None => quote! { ColumnType::Decimal(None) },
            },
            ColumnType::DateTime(s) => match s {
                Some(s) => quote! { ColumnType::DateTime(Some(#s)) },
                None => quote! { ColumnType::DateTime(None) },
            },
            ColumnType::Timestamp(s) => match s {
                Some(s) => quote! { ColumnType::Timestamp(Some(#s)) },
                None => quote! { ColumnType::Timestamp(None) },
            },
            ColumnType::Time(s) => match s {
                Some(s) => quote! { ColumnType::Time(Some(#s)) },
                None => quote! { ColumnType::Time(None) },
            },
            ColumnType::Date => quote! { ColumnType::Date },
            ColumnType::Binary(s) => match s {
                Some(s) => quote! { ColumnType::Binary(Some(#s)) },
                None => quote! { ColumnType::Binary(None) },
            },
            ColumnType::Boolean => quote! { ColumnType::Boolean },
            ColumnType::Money(s) => match s {
                Some((s1, s2)) => quote! { ColumnType::Money(Some((#s1, #s2))) },
                None => quote! { ColumnType::Money(None) },
            },
            ColumnType::Json => quote! { ColumnType::Json },
            ColumnType::JsonBinary => quote! { ColumnType::JsonBinary },
            ColumnType::Custom(s) => {
                let s = s.to_string();
                quote! { ColumnType::Custom(sea_query::SeaRc::new(sea_query::Alias::new(#s))) }
            }
        }
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
        Self {
            name,
            col_type,
            auto_increment,
            not_null,
        }
    }
}
