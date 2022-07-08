use crate::util::escape_rust_keyword;
use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use sea_query::{BlobSize, ColumnDef, ColumnSpec, ColumnType};
use std::fmt::Write as FmtWrite;

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
        format_ident!("{}", escape_rust_keyword(self.name.to_snake_case()))
    }

    pub fn get_name_camel_case(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.name.to_camel_case()))
    }

    pub fn is_snake_case_name(&self) -> bool {
        self.name.to_snake_case() == self.name
    }

    pub fn get_rs_type(&self) -> TokenStream {
        #[allow(unreachable_patterns)]
        let ident: TokenStream = match &self.col_type {
            ColumnType::Char(_)
            | ColumnType::String(_)
            | ColumnType::Text
            | ColumnType::Custom(_) => "String".to_owned(),
            ColumnType::TinyInteger(_) => "i8".to_owned(),
            ColumnType::SmallInteger(_) => "i16".to_owned(),
            ColumnType::Integer(_) => "i32".to_owned(),
            ColumnType::BigInteger(_) => "i64".to_owned(),
            ColumnType::TinyUnsigned(_) => "u8".to_owned(),
            ColumnType::SmallUnsigned(_) => "u16".to_owned(),
            ColumnType::Unsigned(_) => "u32".to_owned(),
            ColumnType::BigUnsigned(_) => "u64".to_owned(),
            ColumnType::Float(_) => "f32".to_owned(),
            ColumnType::Double(_) => "f64".to_owned(),
            ColumnType::Json | ColumnType::JsonBinary => "Json".to_owned(),
            ColumnType::Date => "Date".to_owned(),
            ColumnType::Time(_) => "Time".to_owned(),
            ColumnType::DateTime(_) => "DateTime".to_owned(),
            ColumnType::Timestamp(_) => "DateTimeUtc".to_owned(),
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
            ColumnType::TinyUnsigned(_) => quote! { ColumnType::TinyUnsigned.def() },
            ColumnType::SmallUnsigned(_) => quote! { ColumnType::SmallUnsigned.def() },
            ColumnType::Unsigned(_) => quote! { ColumnType::Unsigned.def() },
            ColumnType::BigUnsigned(_) => quote! { ColumnType::BigUnsigned.def() },
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
            ColumnType::Binary(BlobSize::Blob(_)) => quote! { ColumnType::Binary.def() },
            ColumnType::Binary(BlobSize::Tiny) => quote! { ColumnType::TinyBinary.def() },
            ColumnType::Binary(BlobSize::Medium) => quote! { ColumnType::MediumBinary.def() },
            ColumnType::Binary(BlobSize::Long) => quote! { ColumnType::LongBinary.def() },
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
            ColumnType::Enum(enum_name, _) => {
                let enum_ident = format_ident!("{}", enum_name.to_camel_case());
                quote! { #enum_ident::db_type() }
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
        col_def
    }

    pub fn get_info(&self) -> String {
        let mut info = String::new();
        let type_info = self.get_rs_type().to_string().replace(' ', "");
        let col_info = self.col_info();
        write!(
            &mut info,
            "Column `{}`: {}{}",
            self.name, type_info, col_info
        )
        .unwrap();
        info
    }

    fn col_info(&self) -> String {
        let mut info = String::new();
        if self.auto_increment {
            write!(&mut info, ", auto_increment").unwrap();
        }
        if self.not_null {
            write!(&mut info, ", not_null").unwrap();
        }
        if self.unique {
            write!(&mut info, ", unique").unwrap();
        }
        info
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
    use sea_query::{Alias, BlobSize, ColumnDef, ColumnType, SeaRc};

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
            make_col!("CakeId", ColumnType::TinyUnsigned(Some(9))),
            make_col!("CakeId", ColumnType::SmallInteger(None)),
            make_col!("CakeId", ColumnType::SmallUnsigned(Some(10))),
            make_col!("CakeId", ColumnType::Integer(None)),
            make_col!("CakeId", ColumnType::Unsigned(Some(11))),
            make_col!("CakeFillingId", ColumnType::BigInteger(None)),
            make_col!("CakeFillingId", ColumnType::BigUnsigned(Some(12))),
            make_col!("cake-filling-id", ColumnType::Float(None)),
            make_col!("CAKE_FILLING_ID", ColumnType::Double(None)),
            make_col!("CAKE-FILLING-ID", ColumnType::Binary(BlobSize::Blob(None))),
            make_col!("CAKE", ColumnType::Boolean),
            make_col!("date", ColumnType::Date),
            make_col!("time", ColumnType::Time(None)),
            make_col!("date_time", ColumnType::DateTime(None)),
            make_col!("timestamp", ColumnType::Timestamp(None)),
            make_col!("timestamp_tz", ColumnType::TimestampWithTimeZone(None)),
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
            "DateTimeUtc",
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
            "ColumnType::TinyUnsigned.def()",
            "ColumnType::SmallInteger.def()",
            "ColumnType::SmallUnsigned.def()",
            "ColumnType::Integer.def()",
            "ColumnType::Unsigned.def()",
            "ColumnType::BigInteger.def()",
            "ColumnType::BigUnsigned.def()",
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
    fn test_get_info() {
        let column: Column = ColumnDef::new(Alias::new("id")).string().to_owned().into();
        assert_eq!(column.get_info().as_str(), "Column `id`: Option<String>");

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(column.get_info().as_str(), "Column `id`: String, not_null");

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .not_null()
            .unique_key()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info().as_str(),
            "Column `id`: String, not_null, unique"
        );

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .not_null()
            .unique_key()
            .auto_increment()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info().as_str(),
            "Column `id`: String, auto_increment, not_null, unique"
        );
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
