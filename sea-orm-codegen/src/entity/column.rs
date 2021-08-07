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
        let ident: TokenStream = match self.col_type {
            ColumnType::Char(_)
            | ColumnType::String(_)
            | ColumnType::Text
            | ColumnType::Time(_)
            | ColumnType::Date
            | ColumnType::Custom(_) => "String",
            ColumnType::TinyInteger(_) => "i8",
            ColumnType::SmallInteger(_) => "i16",
            ColumnType::Integer(_) => "i32",
            ColumnType::BigInteger(_) => "i64",
            ColumnType::Float(_) => "f32",
            ColumnType::Double(_) => "f64",
            ColumnType::Json | ColumnType::JsonBinary => "Json",
            ColumnType::DateTime(_) | ColumnType::Timestamp(_) => "DateTime",
            ColumnType::Decimal(_) | ColumnType::Money(_) => "Decimal",
            ColumnType::Uuid => "Uuid",
            ColumnType::Binary(_) => "Vec<u8>",
            ColumnType::Boolean => "bool",
        }
        .parse()
        .unwrap();
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
            ColumnType::TinyInteger(_) => quote! { ColumnType::TinyInteger.def() },
            ColumnType::SmallInteger(_) => quote! { ColumnType::SmallInteger.def() },
            ColumnType::Integer(_) => quote! { ColumnType::Integer.def() },
            ColumnType::BigInteger(_) => quote! { ColumnType::BigInteger.def() },
            ColumnType::Float(_) => quote! { ColumnType::Float.def() },
            ColumnType::Double(_) => quote! { ColumnType::Double.def() },
            ColumnType::Decimal(s) => match s {
                Some((s1, s2)) => quote! { ColumnType::Decimal(Some((#s1, #s2))).def() },
                None => quote! { ColumnType::Decimal(None).def() },
            },
            ColumnType::DateTime(_) => quote! { ColumnType::DateTime.def() },
            ColumnType::Timestamp(_) => quote! { ColumnType::Timestamp.def() },
            ColumnType::Time(_) => quote! { ColumnType::Time.def() },
            ColumnType::Date => quote! { ColumnType::Date.def() },
            ColumnType::Binary(_) => quote! { ColumnType::Binary.def() },
            ColumnType::Boolean => quote! { ColumnType::Boolean.def() },
            ColumnType::Money(s) => match s {
                Some((s1, s2)) => quote! { ColumnType::Money(Some((#s1, #s2))).def() },
                None => quote! { ColumnType::Money(None).def() },
            },
            ColumnType::Json => quote! { ColumnType::Json.def() },
            ColumnType::JsonBinary => quote! { ColumnType::JsonBinary.def() },
            ColumnType::Uuid => quote! { ColumnType::Uuid.def() },
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

impl From<ColumnDef> for Column {
    fn from(col_def: ColumnDef) -> Self {
        (&col_def).into()
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

#[cfg(test)]
mod tests {
    use crate::Column;
    use proc_macro2::TokenStream;
    use quote::quote;
    use sea_query::{Alias, ColumnDef, ColumnType, SeaRc};

    fn setup() -> Vec<Column> {
        macro_rules! make_col {
            ($name:expr, $col_type:expr) => {
                Column {
                    name: $name.to_owned(),
                    col_type: $col_type,
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                }
            };
        }
        vec![
            make_col!("id", ColumnType::String(Some(255))),
            make_col!(
                "cake_id",
                ColumnType::Custom(SeaRc::new(Alias::new("cus_col")))
            ),
            make_col!("CakeId", ColumnType::TinyInteger(None)),
            make_col!("CakeId", ColumnType::SmallInteger(None)),
            make_col!("CakeId", ColumnType::Integer(Some(11))),
            make_col!("CakeFillingId", ColumnType::BigInteger(None)),
            make_col!("cake-filling-id", ColumnType::Float(None)),
            make_col!("CAKE_FILLING_ID", ColumnType::Double(None)),
            make_col!("CAKE-FILLING-ID", ColumnType::Binary(None)),
            make_col!("CAKE", ColumnType::Boolean),
        ]
    }

    #[test]
    fn test_get_name_snake_case() {
        let columns = setup();
        let snack_cases = vec![
            "id",
            "cake_id",
            "cake_id",
            "cake_id",
            "cake_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake",
        ];
        for (col, snack_case) in columns.into_iter().zip(snack_cases) {
            assert_eq!(col.get_name_snake_case().to_string(), snack_case);
        }
    }

    #[test]
    fn test_get_name_camel_case() {
        let columns = setup();
        let camel_cases = vec![
            "Id",
            "CakeId",
            "CakeId",
            "CakeId",
            "CakeId",
            "CakeFillingId",
            "CakeFillingId",
            "CakeFillingId",
            "CakeFillingId",
            "Cake",
        ];
        for (col, camel_case) in columns.into_iter().zip(camel_cases) {
            assert_eq!(col.get_name_camel_case().to_string(), camel_case);
        }
    }

    #[test]
    fn test_get_rs_type() {
        let columns = setup();
        let rs_types = vec![
            "String", "String", "i8", "i16", "i32", "i64", "f32", "f64", "Vec<u8>", "bool",
        ];
        for (mut col, rs_type) in columns.into_iter().zip(rs_types) {
            let rs_type: TokenStream = rs_type.parse().unwrap();

            col.not_null = true;
            assert_eq!(col.get_rs_type().to_string(), quote!(#rs_type).to_string());

            col.not_null = false;
            assert_eq!(
                col.get_rs_type().to_string(),
                quote!(Option<#rs_type>).to_string()
            );
        }
    }

    #[test]
    fn test_get_def() {
        let columns = setup();
        let col_defs = vec![
            "ColumnType::String(Some(255u32)).def()",
            "ColumnType::Custom(\"cus_col\".to_owned()).def()",
            "ColumnType::TinyInteger.def()",
            "ColumnType::SmallInteger.def()",
            "ColumnType::Integer.def()",
            "ColumnType::BigInteger.def()",
            "ColumnType::Float.def()",
            "ColumnType::Double.def()",
            "ColumnType::Binary.def()",
            "ColumnType::Boolean.def()",
        ];
        for (mut col, col_def) in columns.into_iter().zip(col_defs) {
            let mut col_def: TokenStream = col_def.parse().unwrap();

            col.not_null = true;
            assert_eq!(col.get_def().to_string(), col_def.to_string());

            col.not_null = false;
            col_def.extend(quote!(.null()));
            assert_eq!(col.get_def().to_string(), col_def.to_string());

            col.unique = true;
            col_def.extend(quote!(.unique()));
            assert_eq!(col.get_def().to_string(), col_def.to_string());
        }
    }

    #[test]
    fn test_from_column_def() {
        let column: Column = ColumnDef::new(Alias::new("id")).string().into();
        assert_eq!(
            column.get_def().to_string(),
            quote! {
                ColumnType::String(None).def().null()
            }
            .to_string()
        );

        let column: Column = ColumnDef::new(Alias::new("id")).string().not_null().into();
        assert!(column.not_null);

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .unique_key()
            .not_null()
            .into();
        assert!(column.unique);
        assert!(column.not_null);

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .auto_increment()
            .unique_key()
            .not_null()
            .into();
        assert!(column.auto_increment);
        assert!(column.unique);
        assert!(column.not_null);
    }
}
