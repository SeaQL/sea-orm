use crate::util::escape_rust_keyword;
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
    pub(crate) unsigned: bool,
}

impl Column {
    pub fn get_name_snake_case(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.name.to_snake_case()))
    }

    pub fn get_name_camel_case(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.name.to_camel_case()))
    }

    pub fn get_rs_type(&self) -> TokenStream {
        #[allow(unreachable_patterns)]
        let ident: TokenStream = match &self.col_type {
            ColumnType::Char(_)
            | ColumnType::String(_)
            | ColumnType::Text
            | ColumnType::Custom(_) => "String".to_owned(),
            ColumnType::TinyInteger(_) => if self.unsigned { "u8" } else { "i8" }.to_owned(),
            ColumnType::SmallInteger(_) => if self.unsigned { "u16" } else { "i16" }.to_owned(),
            ColumnType::Integer(_) => if self.unsigned { "u32" } else { "i32" }.to_owned(),
            ColumnType::BigInteger(_) => if self.unsigned { "u64" } else { "i64" }.to_owned(),
            ColumnType::Float(_) => "f32".to_owned(),
            ColumnType::Double(_) => "f64".to_owned(),
            ColumnType::Json | ColumnType::JsonBinary => "Json".to_owned(),
            ColumnType::Date => "Date".to_owned(),
            ColumnType::Time(_) => "Time".to_owned(),
            ColumnType::DateTime(_) | ColumnType::Timestamp(_) => "DateTime".to_owned(),
            ColumnType::TimestampWithTimeZone(_) => "DateTimeWithTimeZone".to_owned(),
            ColumnType::Decimal(_) | ColumnType::Money(_) => "Decimal".to_owned(),
            ColumnType::Uuid => "Uuid".to_owned(),
            ColumnType::Binary(_) => "Vec<u8>".to_owned(),
            ColumnType::Boolean => "bool".to_owned(),
            ColumnType::Enum(name, _) => name.to_camel_case(),
            _ => unimplemented!(),
        }
        .parse()
        .unwrap();
        match self.not_null {
            true => quote! { #ident },
            false => quote! { Option<#ident> },
        }
    }

    pub fn get_col_type_attrs(&self) -> Option<TokenStream> {
        let col_type = match &self.col_type {
            ColumnType::Float(Some(l)) => Some(format!("Float(Some({}))", l)),
            ColumnType::Double(Some(l)) => Some(format!("Double(Some({}))", l)),
            ColumnType::Decimal(Some((p, s))) => Some(format!("Decimal(Some(({}, {})))", p, s)),
            ColumnType::Money(Some((p, s))) => Some(format!("Money(Some({}, {}))", p, s)),
            ColumnType::Text => Some("Text".to_owned()),
            ColumnType::Custom(iden) => {
                Some(format!("Custom(\"{}\".to_owned())", iden.to_string()))
            }
            _ => None,
        };
        col_type.map(|ty| quote! { column_type = #ty })
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
            ColumnType::TimestampWithTimeZone(_) => {
                quote! { ColumnType::TimestampWithTimeZone.def() }
            }
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
            #[allow(unreachable_patterns)]
            _ => unimplemented!(),
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
        if self.unsigned {
            col_def.extend(quote! {
                .unsigned()
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
        let auto_increment = col_def
            .get_column_spec()
            .iter()
            .any(|spec| matches!(spec, ColumnSpec::AutoIncrement));
        let not_null = col_def
            .get_column_spec()
            .iter()
            .any(|spec| matches!(spec, ColumnSpec::NotNull));
        let unique = col_def
            .get_column_spec()
            .iter()
            .any(|spec| matches!(spec, ColumnSpec::UniqueKey));
        let unsigned = col_def
            .get_column_spec()
            .iter()
            .any(|spec| matches!(spec, ColumnSpec::Unsigned));
        Self {
            name,
            col_type,
            auto_increment,
            not_null,
            unique,
            unsigned,
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
            ($name:expr, $col_type:expr, $unsigned:expr) => {
                Column {
                    name: $name.to_owned(),
                    col_type: $col_type,
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                    unsigned: $unsigned,
                }
            };
        }
        vec![
            make_col!("id", ColumnType::String(Some(255)), false),
            make_col!(
                "cake_id",
                ColumnType::Custom(SeaRc::new(Alias::new("cus_col"))),
                false
            ),
            make_col!("CakeId", ColumnType::TinyInteger(None), false),
            make_col!("CakeId", ColumnType::TinyInteger(Some(9)), true),
            make_col!("CakeId", ColumnType::SmallInteger(None), false),
            make_col!("CakeId", ColumnType::SmallInteger(Some(10)), true),
            make_col!("CakeId", ColumnType::Integer(None), false),
            make_col!("CakeId", ColumnType::Integer(Some(11)), true),
            make_col!("CakeFillingId", ColumnType::BigInteger(None), false),
            make_col!("CakeFillingId", ColumnType::BigInteger(Some(12)), true),
            make_col!("cake-filling-id", ColumnType::Float(None), false),
            make_col!("CAKE_FILLING_ID", ColumnType::Double(None), false),
            make_col!("CAKE-FILLING-ID", ColumnType::Binary(None), false),
            make_col!("CAKE", ColumnType::Boolean, false),
            make_col!("date", ColumnType::Date, false),
            make_col!("time", ColumnType::Time(None), false),
            make_col!("date_time", ColumnType::DateTime(None), false),
            make_col!("timestamp", ColumnType::Timestamp(None), false),
            make_col!(
                "timestamp_tz",
                ColumnType::TimestampWithTimeZone(None),
                false
            ),
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
            "cake_id",
            "cake_id",
            "cake_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake_filling_id",
            "cake",
            "date",
            "time",
            "date_time",
            "timestamp",
            "timestamp_tz",
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
            "CakeId",
            "CakeId",
            "CakeId",
            "CakeFillingId",
            "CakeFillingId",
            "CakeFillingId",
            "CakeFillingId",
            "CakeFillingId",
            "Cake",
            "Date",
            "Time",
            "DateTime",
            "Timestamp",
            "TimestampTz",
        ];
        for (col, camel_case) in columns.into_iter().zip(camel_cases) {
            assert_eq!(col.get_name_camel_case().to_string(), camel_case);
        }
    }

    #[test]
    fn test_get_rs_type() {
        let columns = setup();
        let rs_types = vec![
            "String",
            "String",
            "i8",
            "u8",
            "i16",
            "u16",
            "i32",
            "u32",
            "i64",
            "u64",
            "f32",
            "f64",
            "Vec<u8>",
            "bool",
            "Date",
            "Time",
            "DateTime",
            "DateTime",
            "DateTimeWithTimeZone",
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
            "ColumnType::TinyInteger.def()",
            "ColumnType::SmallInteger.def()",
            "ColumnType::SmallInteger.def()",
            "ColumnType::Integer.def()",
            "ColumnType::Integer.def()",
            "ColumnType::BigInteger.def()",
            "ColumnType::BigInteger.def()",
            "ColumnType::Float.def()",
            "ColumnType::Double.def()",
            "ColumnType::Binary.def()",
            "ColumnType::Boolean.def()",
            "ColumnType::Date.def()",
            "ColumnType::Time.def()",
            "ColumnType::DateTime.def()",
            "ColumnType::Timestamp.def()",
            "ColumnType::TimestampWithTimeZone.def()",
        ];
        for (mut col, col_def) in columns.into_iter().zip(col_defs) {
            let mut col_def: TokenStream = col_def.parse().unwrap();

            col.unsigned = false;
            col.not_null = true;
            assert_eq!(col.get_def().to_string(), col_def.to_string());

            col.not_null = false;
            col_def.extend(quote!(.null()));
            assert_eq!(col.get_def().to_string(), col_def.to_string());

            col.unique = true;
            col_def.extend(quote!(.unique()));
            assert_eq!(col.get_def().to_string(), col_def.to_string());

            col.unsigned = true;
            col_def.extend(quote!(.unsigned()));
            assert_eq!(col.get_def().to_string(), col_def.to_string());
        }
    }

    #[test]
    fn test_from_column_def() {
        let column: Column = ColumnDef::new(Alias::new("id")).string().to_owned().into();
        assert_eq!(
            column.get_def().to_string(),
            quote! {
                ColumnType::String(None).def().null()
            }
            .to_string()
        );

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .not_null()
            .to_owned()
            .into();
        assert!(column.not_null);

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .unique_key()
            .not_null()
            .to_owned()
            .into();
        assert!(column.unique);
        assert!(column.not_null);

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .auto_increment()
            .unique_key()
            .not_null()
            .to_owned()
            .into();
        assert!(column.auto_increment);
        assert!(column.unique);
        assert!(column.not_null);
    }
}
