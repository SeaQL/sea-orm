use crate::{BigIntegerType, DateTimeCrate, util::escape_rust_keyword};
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use sea_query::{ColumnDef, ColumnType, StringLen};
use std::fmt::Write as FmtWrite;

#[derive(Debug, Clone)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) col_type: ColumnType,
    pub(crate) auto_increment: bool,
    pub(crate) not_null: bool,
    pub(crate) unique: bool,
    pub(crate) unique_key: Option<String>,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ColumnOption {
    pub(crate) date_time_crate: DateTimeCrate,
    pub(crate) big_integer_type: BigIntegerType,
}

impl Column {
    pub fn get_name_snake_case(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.name.to_snake_case()))
    }

    pub fn get_name_camel_case(&self) -> Ident {
        format_ident!("{}", escape_rust_keyword(self.name.to_upper_camel_case()))
    }

    pub fn is_snake_case_name(&self) -> bool {
        self.name.to_snake_case() == self.name
    }

    pub fn get_rs_type(&self, opt: &ColumnOption) -> TokenStream {
        fn write_rs_type(col_type: &ColumnType, opt: &ColumnOption) -> String {
            #[allow(unreachable_patterns)]
            match col_type {
                ColumnType::Char(_)
                | ColumnType::String(_)
                | ColumnType::Text
                | ColumnType::Custom(_) => "String".to_owned(),
                ColumnType::TinyInteger => "i8".to_owned(),
                ColumnType::SmallInteger => "i16".to_owned(),
                ColumnType::Integer => "i32".to_owned(),
                ColumnType::BigInteger => match opt.big_integer_type {
                    BigIntegerType::I64 => "i64",
                    BigIntegerType::I32 => "i32",
                }
                .to_owned(),
                ColumnType::TinyUnsigned => "u8".to_owned(),
                ColumnType::SmallUnsigned => "u16".to_owned(),
                ColumnType::Unsigned => "u32".to_owned(),
                ColumnType::BigUnsigned => "u64".to_owned(),
                ColumnType::Float => "f32".to_owned(),
                ColumnType::Double => "f64".to_owned(),
                ColumnType::Json | ColumnType::JsonBinary => "Json".to_owned(),
                ColumnType::Date => match opt.date_time_crate {
                    DateTimeCrate::Chrono => "Date".to_owned(),
                    DateTimeCrate::Time => "TimeDate".to_owned(),
                },
                ColumnType::Time => match opt.date_time_crate {
                    DateTimeCrate::Chrono => "Time".to_owned(),
                    DateTimeCrate::Time => "TimeTime".to_owned(),
                },
                ColumnType::DateTime => match opt.date_time_crate {
                    DateTimeCrate::Chrono => "DateTime".to_owned(),
                    DateTimeCrate::Time => "TimeDateTime".to_owned(),
                },
                ColumnType::Timestamp => match opt.date_time_crate {
                    DateTimeCrate::Chrono => "DateTimeUtc".to_owned(),
                    DateTimeCrate::Time => "TimeDateTime".to_owned(),
                },
                ColumnType::TimestampWithTimeZone => match opt.date_time_crate {
                    DateTimeCrate::Chrono => "DateTimeWithTimeZone".to_owned(),
                    DateTimeCrate::Time => "TimeDateTimeWithTimeZone".to_owned(),
                },
                ColumnType::Decimal(_) | ColumnType::Money(_) => "Decimal".to_owned(),
                ColumnType::Uuid => "Uuid".to_owned(),
                ColumnType::Binary(_) | ColumnType::VarBinary(_) | ColumnType::Blob => {
                    "Vec<u8>".to_owned()
                }
                ColumnType::Boolean => "bool".to_owned(),
                ColumnType::Enum { name, .. } => name.to_string().to_upper_camel_case(),
                ColumnType::Array(column_type) => {
                    format!("Vec<{}>", write_rs_type(column_type, opt))
                }
                ColumnType::Vector(_) => "PgVector".to_owned(),
                ColumnType::Bit(None | Some(1)) => "bool".to_owned(),
                ColumnType::Bit(_) | ColumnType::VarBit(_) => "Vec<u8>".to_owned(),
                ColumnType::Year => "i32".to_owned(),
                ColumnType::Cidr | ColumnType::Inet => "IpNetwork".to_owned(),
                ColumnType::Interval(_, _) | ColumnType::MacAddr | ColumnType::LTree => {
                    "String".to_owned()
                }
                _ => unimplemented!(),
            }
        }
        let ident: TokenStream = write_rs_type(&self.col_type, opt).parse().unwrap();
        match self.not_null {
            true => quote! { #ident },
            false => quote! { Option<#ident> },
        }
    }

    pub fn get_col_type_attrs(&self) -> Option<TokenStream> {
        let col_type = match &self.col_type {
            ColumnType::Float => Some("Float".to_owned()),
            ColumnType::Double => Some("Double".to_owned()),
            ColumnType::Decimal(Some((p, s))) => Some(format!("Decimal(Some(({p}, {s})))")),
            ColumnType::Money(Some((p, s))) => Some(format!("Money(Some({p}, {s}))")),
            ColumnType::Text => Some("Text".to_owned()),
            ColumnType::JsonBinary => Some("JsonBinary".to_owned()),
            ColumnType::Custom(iden) => {
                let ty = format!("custom(\"{iden}\")");
                return Some(quote! ( ignore, column_type = #ty, select_as = "text" ));
            }
            ColumnType::Binary(s) => Some(format!("Binary({s})")),
            ColumnType::VarBinary(s) => match s {
                StringLen::N(s) => Some(format!("VarBinary(StringLen::N({s}))")),
                StringLen::None => Some("VarBinary(StringLen::None)".to_owned()),
                StringLen::Max => Some("VarBinary(StringLen::Max)".to_owned()),
            },
            ColumnType::Blob => Some("Blob".to_owned()),
            ColumnType::Cidr => Some("Cidr".to_owned()),
            _ => None,
        };
        col_type.map(|ty| quote! { column_type = #ty })
    }

    pub fn get_def(&self) -> TokenStream {
        fn write_col_def(col_type: &ColumnType) -> TokenStream {
            match col_type {
                ColumnType::Char(s) => match s {
                    Some(s) => quote! { ColumnType::Char(Some(#s)) },
                    None => quote! { ColumnType::Char(None) },
                },
                ColumnType::String(s) => match s {
                    StringLen::N(s) => quote! { ColumnType::String(StringLen::N(#s)) },
                    StringLen::None => quote! { ColumnType::String(StringLen::None) },
                    StringLen::Max => quote! { ColumnType::String(StringLen::Max) },
                },
                ColumnType::Text => quote! { ColumnType::Text },
                ColumnType::TinyInteger => quote! { ColumnType::TinyInteger },
                ColumnType::SmallInteger => quote! { ColumnType::SmallInteger },
                ColumnType::Integer => quote! { ColumnType::Integer },
                ColumnType::BigInteger => quote! { ColumnType::BigInteger },
                ColumnType::TinyUnsigned => quote! { ColumnType::TinyUnsigned },
                ColumnType::SmallUnsigned => quote! { ColumnType::SmallUnsigned },
                ColumnType::Unsigned => quote! { ColumnType::Unsigned },
                ColumnType::BigUnsigned => quote! { ColumnType::BigUnsigned },
                ColumnType::Float => quote! { ColumnType::Float },
                ColumnType::Double => quote! { ColumnType::Double },
                ColumnType::Decimal(s) => match s {
                    Some((s1, s2)) => quote! { ColumnType::Decimal(Some((#s1, #s2))) },
                    None => quote! { ColumnType::Decimal(None) },
                },
                ColumnType::DateTime => quote! { ColumnType::DateTime },
                ColumnType::Timestamp => quote! { ColumnType::Timestamp },
                ColumnType::TimestampWithTimeZone => {
                    quote! { ColumnType::TimestampWithTimeZone }
                }
                ColumnType::Time => quote! { ColumnType::Time },
                ColumnType::Date => quote! { ColumnType::Date },
                ColumnType::Binary(s) => {
                    quote! { ColumnType::Binary(#s) }
                }
                ColumnType::VarBinary(s) => match s {
                    StringLen::N(s) => quote! { ColumnType::VarBinary(StringLen::N(#s)) },
                    StringLen::None => quote! { ColumnType::VarBinary(StringLen::None) },
                    StringLen::Max => quote! { ColumnType::VarBinary(StringLen::Max) },
                },
                ColumnType::Blob => quote! { ColumnType::Blob },
                ColumnType::Boolean => quote! { ColumnType::Boolean },
                ColumnType::Money(s) => match s {
                    Some((s1, s2)) => quote! { ColumnType::Money(Some((#s1, #s2))) },
                    None => quote! { ColumnType::Money(None) },
                },
                ColumnType::Json => quote! { ColumnType::Json },
                ColumnType::JsonBinary => quote! { ColumnType::JsonBinary },
                ColumnType::Uuid => quote! { ColumnType::Uuid },
                ColumnType::Cidr => quote! { ColumnType::Cidr },
                ColumnType::Inet => quote! { ColumnType::Inet },
                ColumnType::Custom(s) => {
                    let s = s.to_string();
                    quote! { ColumnType::custom(#s) }
                }
                ColumnType::Enum { name, .. } => {
                    let enum_ident = format_ident!("{}", name.to_string().to_upper_camel_case());
                    quote! {
                        #enum_ident::db_type()
                            .get_column_type()
                            .to_owned()
                    }
                }
                ColumnType::Array(column_type) => {
                    let column_type = write_col_def(column_type);
                    quote! { ColumnType::Array(RcOrArc::new(#column_type)) }
                }
                ColumnType::Vector(size) => match size {
                    Some(size) => quote! { ColumnType::Vector(Some(#size)) },
                    None => quote! { ColumnType::Vector(None) },
                },
                #[allow(unreachable_patterns)]
                _ => unimplemented!(),
            }
        }
        let mut col_def = write_col_def(&self.col_type);
        col_def.extend(quote! {
            .def()
        });
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

    pub fn get_info(&self, opt: &ColumnOption) -> String {
        let mut info = String::new();
        let type_info = self.get_rs_type(opt).to_string().replace(' ', "");
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

    pub fn get_serde_attribute(
        &self,
        is_primary_key: bool,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
    ) -> TokenStream {
        if self.name.starts_with('_') && serde_skip_hidden_column {
            quote! {
                #[serde(skip)]
            }
        } else if serde_skip_deserializing_primary_key && is_primary_key {
            quote! {
                #[serde(skip_deserializing)]
            }
        } else {
            quote! {}
        }
    }

    pub fn get_inner_col_type(&self) -> &ColumnType {
        match &self.col_type {
            ColumnType::Array(inner_col_type) => inner_col_type.as_ref(),
            _ => &self.col_type,
        }
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
        let auto_increment = col_def.get_column_spec().auto_increment;
        let not_null = match col_def.get_column_spec().nullable {
            Some(nullable) => !nullable,
            None => false,
        };
        let unique = col_def.get_column_spec().unique;
        Self {
            name,
            col_type,
            auto_increment,
            not_null,
            unique,
            unique_key: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Column, ColumnOption, DateTimeCrate};
    use proc_macro2::TokenStream;
    use quote::quote;
    use sea_query::{Alias, ColumnDef, ColumnType, SeaRc, StringLen};

    fn date_time_crate_chrono() -> ColumnOption {
        ColumnOption {
            date_time_crate: DateTimeCrate::Chrono,
            big_integer_type: Default::default(),
        }
    }

    fn date_time_crate_time() -> ColumnOption {
        ColumnOption {
            date_time_crate: DateTimeCrate::Time,
            big_integer_type: Default::default(),
        }
    }

    fn setup() -> Vec<Column> {
        macro_rules! make_col {
            ($name:expr, $col_type:expr) => {
                Column {
                    name: $name.to_owned(),
                    col_type: $col_type,
                    auto_increment: false,
                    not_null: false,
                    unique: false,
                    unique_key: None,
                }
            };
        }
        vec![
            make_col!("id", ColumnType::String(StringLen::N(255))),
            make_col!("id", ColumnType::String(StringLen::None)),
            make_col!(
                "cake_id",
                ColumnType::Custom(SeaRc::new(Alias::new("cus_col")))
            ),
            make_col!("CakeId", ColumnType::TinyInteger),
            make_col!("CakeId", ColumnType::TinyUnsigned),
            make_col!("CakeId", ColumnType::SmallInteger),
            make_col!("CakeId", ColumnType::SmallUnsigned),
            make_col!("CakeId", ColumnType::Integer),
            make_col!("CakeId", ColumnType::Unsigned),
            make_col!("CakeFillingId", ColumnType::BigInteger),
            make_col!("CakeFillingId", ColumnType::BigUnsigned),
            make_col!("cake-filling-id", ColumnType::Float),
            make_col!("CAKE_FILLING_ID", ColumnType::Double),
            make_col!("CAKE-FILLING-ID", ColumnType::Binary(10)),
            make_col!("CAKE-FILLING-ID", ColumnType::VarBinary(StringLen::None)),
            make_col!("CAKE-FILLING-ID", ColumnType::VarBinary(StringLen::N(10))),
            make_col!("CAKE-FILLING-ID", ColumnType::VarBinary(StringLen::Max)),
            make_col!("CAKE", ColumnType::Boolean),
            make_col!("date", ColumnType::Date),
            make_col!("time", ColumnType::Time),
            make_col!("date_time", ColumnType::DateTime),
            make_col!("timestamp", ColumnType::Timestamp),
            make_col!("timestamp_tz", ColumnType::TimestampWithTimeZone),
        ]
    }

    #[test]
    fn test_get_name_snake_case() {
        let columns = setup();
        let snack_cases = vec![
            "id",
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
    fn test_get_rs_type_with_chrono() {
        let columns = setup();
        let rs_types = vec![
            "String",
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
            "Vec<u8>",
            "Vec<u8>",
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
            assert_eq!(
                col.get_rs_type(&date_time_crate_chrono()).to_string(),
                quote!(#rs_type).to_string()
            );

            col.not_null = false;
            assert_eq!(
                col.get_rs_type(&date_time_crate_chrono()).to_string(),
                quote!(Option<#rs_type>).to_string()
            );
        }
    }

    #[test]
    fn test_get_rs_type_with_time() {
        let columns = setup();
        let rs_types = vec![
            "String",
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
            "Vec<u8>",
            "Vec<u8>",
            "Vec<u8>",
            "bool",
            "TimeDate",
            "TimeTime",
            "TimeDateTime",
            "TimeDateTime",
            "TimeDateTimeWithTimeZone",
        ];
        for (mut col, rs_type) in columns.into_iter().zip(rs_types) {
            let rs_type: TokenStream = rs_type.parse().unwrap();

            col.not_null = true;
            assert_eq!(
                col.get_rs_type(&date_time_crate_time()).to_string(),
                quote!(#rs_type).to_string()
            );

            col.not_null = false;
            assert_eq!(
                col.get_rs_type(&date_time_crate_time()).to_string(),
                quote!(Option<#rs_type>).to_string()
            );
        }
    }

    #[test]
    fn test_get_def() {
        let columns = setup();
        let col_defs = vec![
            "ColumnType::String(StringLen::N(255u32)).def()",
            "ColumnType::String(StringLen::None).def()",
            "ColumnType::custom(\"cus_col\").def()",
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
            "ColumnType::Binary(10u32).def()",
            "ColumnType::VarBinary(StringLen::None).def()",
            "ColumnType::VarBinary(StringLen::N(10u32)).def()",
            "ColumnType::VarBinary(StringLen::Max).def()",
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
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `id`: Option<String>"
        );

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `id`: String, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("id"))
            .string()
            .not_null()
            .unique_key()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
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
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `id`: String, auto_increment, not_null, unique"
        );

        let column: Column = ColumnDef::new(Alias::new("date_field"))
            .date()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `date_field`: Date, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("date_field"))
            .date()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_time()).as_str(),
            "Column `date_field`: TimeDate, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("time_field"))
            .time()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `time_field`: Time, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("time_field"))
            .time()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_time()).as_str(),
            "Column `time_field`: TimeTime, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("date_time_field"))
            .date_time()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `date_time_field`: DateTime, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("date_time_field"))
            .date_time()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_time()).as_str(),
            "Column `date_time_field`: TimeDateTime, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("timestamp_field"))
            .timestamp()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `timestamp_field`: DateTimeUtc, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("timestamp_field"))
            .timestamp()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_time()).as_str(),
            "Column `timestamp_field`: TimeDateTime, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("timestamp_with_timezone_field"))
            .timestamp_with_time_zone()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_chrono()).as_str(),
            "Column `timestamp_with_timezone_field`: DateTimeWithTimeZone, not_null"
        );

        let column: Column = ColumnDef::new(Alias::new("timestamp_with_timezone_field"))
            .timestamp_with_time_zone()
            .not_null()
            .to_owned()
            .into();
        assert_eq!(
            column.get_info(&date_time_crate_time()).as_str(),
            "Column `timestamp_with_timezone_field`: TimeDateTimeWithTimeZone, not_null"
        );
    }

    #[test]
    fn test_from_column_def() {
        let column: Column = ColumnDef::new(Alias::new("id")).string().to_owned().into();
        assert_eq!(
            column.get_def().to_string(),
            quote! {
                ColumnType::String(StringLen::None).def().null()
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
