use std::{process, rc::Rc};

use colorful::Colorful;
use heck::CamelCase;
use quote::ToTokens;
use sea_schema::{
    postgres::{
        def::{ColumnInfo, Type as PgType},
        discovery::SchemaDiscovery,
    },
    sea_query::Alias,
};
use sqlx::PgPool;
use syn::{punctuated::Punctuated, Field};

#[derive(Debug)]
pub enum Error {
    SqlxError(sqlx::Error),
    MissingDatabaseUrl,
}

fn discover_columns_sync(schema: &str, table: &str) -> Result<Vec<ColumnInfo>, Error> {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").map_err(|_| Error::MissingDatabaseUrl)?;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let connection = PgPool::connect(&db_url).await.map_err(Error::SqlxError)?;
        let schema_discovery = SchemaDiscovery::new(connection, schema);
        let columns = schema_discovery
            .discover_columns(Rc::new(Alias::new(schema)), Rc::new(Alias::new(table)))
            .await;
        Ok(columns)
    })
}

fn compare_field_name(field_name: &syn::Ident, db_name: &str) -> bool {
    field_name.to_string().to_camel_case() == db_name.to_camel_case()
}

fn validate_model_type(ty: syn::Type, db_type: &PgType) -> (bool, Option<&'static str>) {
    let ty_string = ty.into_token_stream().to_string();
    let ty_str = ty_string.as_str();
    match db_type {
        PgType::SmallInt | PgType::SmallSerial => (ty_str == "i16", Some("i16")),
        PgType::Integer | PgType::Serial => (ty_str == "i32", Some("i32")),
        PgType::BigInt | PgType::BigSerial => (ty_str == "i64", Some("i64")),
        PgType::Decimal(_) | PgType::Numeric(_) => (
            ["rust_decimal::Decimal", "Decimal"].contains(&ty_str),
            Some("rust_decimal::Decimal"),
        ),
        PgType::Real => (ty_str == "f32", Some("f32")),
        PgType::DoublePrecision => (ty_str == "f64", Some("f64")),
        // PgType::Money => {}
        PgType::Varchar(_) | PgType::Char(_) | PgType::Text => (
            ["std::string::String", "string::String", "String"].contains(&ty_str),
            Some("String"),
        ),
        PgType::Bytea => (
            ["std::vec::Vec < u8 >", "vec::Vec < u8 >", "Vec < u8 >"].contains(&ty_str),
            Some("Vec<u8>"),
        ),
        PgType::Timestamp(_) => (
            ["DateTime", "chrono::NaiveDateTime", "NaiveDateTime"].contains(&ty_str),
            Some("DateTime"),
        ),
        PgType::TimestampWithTimeZone(_) => (
            [
                "DateTimeWithTimeZone",
                "chrono::DateTime < chrono::FixedOffset >",
                "chrono::DateTime < FixedOffset >",
                "DateTime < chrono::FixedOffset >",
                "DateTime < FixedOffset >",
            ]
            .contains(&ty_str),
            Some("DateTimeWithTimeZone"),
        ),
        PgType::Date => (
            ["chrono::NativeDate", "NativeDate"].contains(&ty_str),
            Some("chrono::NativeDate"),
        ),
        PgType::Time(_) | PgType::TimeWithTimeZone(_) => (
            ["chrono::NaiveTime", "NaiveTime"].contains(&ty_str),
            Some("chrono::NativeTime"),
        ),
        // PgType::Interval(_) => {}
        PgType::Boolean => (ty_str == "bool", Some("bool")),
        PgType::Json => (
            ["Json", "serde_json::Value"].contains(&ty_str),
            Some("Json"),
        ),
        PgType::Uuid => (["Uuid", "uuid::Uuid"].contains(&ty_str), Some("Uuid")),
        _ => (false, None),
    }
}

fn print_warnings(msg: &str, warnings: &[impl AsRef<str> + std::fmt::Display]) {
    if !warnings.is_empty() {
        println!("{}: {}", "warning".yellow(), msg.bold());
        for warning in warnings {
            println!("    - {}", warning);
        }
        println!();
    }
}

fn print_errors(msg: &str, errors: &[impl AsRef<str> + std::fmt::Display]) {
    if !errors.is_empty() {
        println!("{}: {}", "error".light_red().bold(), msg.bold());
        for error in errors {
            println!("    - {}", error);
        }
        println!();
        process::exit(1);
    }
}

pub fn validate_fields<P>(
    schema: &str,
    table: &str,
    fields: &Punctuated<Field, P>,
) -> Result<(), Error> {
    let columns = discover_columns_sync(schema, table)?;

    let missing_columns = columns.iter().filter(|col| {
        !fields
            .iter()
            .any(|field| compare_field_name(field.ident.as_ref().unwrap(), &col.name))
    });

    let mut missing_column_warnings = Vec::new();
    for missing_column in missing_columns {
        missing_column_warnings.push(format!(
            "{}{}",
            missing_column.name.as_str(),
            format!(
                ": {}",
                format!("{:?}", missing_column.col_type)
                    .split('(')
                    .next()
                    .unwrap()
                    .trim()
            )
            .dark_gray()
        ));
    }

    let mut unknown_column_warnings = Vec::new();
    let mut unknown_column_type_warnings = Vec::new();
    let mut unknown_column_type_errors = Vec::new();
    for field in fields {
        let db_column = match columns
            .iter()
            .find(|col| compare_field_name(field.ident.as_ref().unwrap(), &col.name))
        {
            Some(db_column) => db_column,
            None => {
                unknown_column_warnings.push(format!("{}", field.ident.as_ref().unwrap()));
                continue;
            }
        };

        match validate_model_type(field.ty.clone(), &db_column.col_type) {
            (false, Some(suggested)) => unknown_column_type_errors.push(format!(
                "{}: {}    expected type {}",
                field.ident.as_ref().unwrap(),
                field.ty.clone().into_token_stream().to_string(),
                suggested.replace(' ', "")
            )),
            (false, None) => unknown_column_type_warnings.push(format!(
                "{}: {}",
                field.ident.as_ref().unwrap(),
                field.ty.clone().into_token_stream().to_string(),
            )),
            _ => (),
        }
    }

    // Print warnings
    print_warnings(
        &format!("missing columns for table `{}`", table),
        &missing_column_warnings,
    );
    print_warnings(
        &format!("missing columns for table `{}`", table),
        &unknown_column_warnings,
    );
    print_warnings(
        &format!("unknown column types for table `{}`", table),
        &unknown_column_type_warnings,
    );
    print_errors(
        &format!("invalid column types for table `{}`", table),
        &unknown_column_type_errors,
    );

    Ok(())
}
