use crate::{error::*, ExecResult, ExecResultHolder, QueryResult, QueryResultRow, Statement};

use sea_query::{Value, ValueType};
use std::{collections::BTreeMap, fmt::Debug};

/// Defines the [ProxyDatabaseTrait] to save the functions
pub trait ProxyDatabaseTrait: Send + Sync + std::fmt::Debug {
    /// Execute a query in the [ProxyDatabase], and return the query results
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr>;

    /// Execute a command in the [ProxyDatabase], and report the number of rows affected
    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr>;

    /// Begin a transaction in the [ProxyDatabase]
    fn begin(&self) {}

    /// Commit a transaction in the [ProxyDatabase]
    fn commit(&self) {}

    /// Rollback a transaction in the [ProxyDatabase]
    fn rollback(&self) {}

    /// Ping the [ProxyDatabase], it should return an error if the database is not available
    fn ping(&self) -> Result<(), DbErr> {
        Ok(())
    }
}

/// The id type for [ProxyExecResult]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ProxyExecResultIdType {
    /// Integer
    Integer(u64),
    /// UUID
    Uuid(uuid::Uuid),
    /// String
    String(String),
    /// Bytes
    Bytes(Vec<u8>),
}

impl Into<u64> for ProxyExecResultIdType {
    fn into(self) -> u64 {
        match self {
            Self::Integer(val) => val,
            Self::String(val) => val.parse().unwrap_or(0),
            Self::Bytes(val) => {
                // It would crash if it's longer than 8 bytes
                if val.len() > 8 {
                    panic!("Bytes is longer than 8 bytes")
                }

                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&val[..8]);
                u64::from_le_bytes(bytes)
            }
            Self::Uuid(_) => panic!("Uuid cannot be converted to u64 that not lose precision"),
        }
    }
}

/// Defines the results obtained from a [ProxyDatabase]
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum ProxyExecResult {
    /// The INSERT statement did not have any value to insert
    #[default]
    Empty,
    /// The INSERT operation did not insert any valid value
    Conflicted,
    /// Successfully inserted
    Inserted(Vec<ProxyExecResultIdType>),
}

impl ProxyExecResult {
    /// Get the last id after `AUTOINCREMENT` is done on the primary key
    ///
    /// # Panics
    ///
    /// The proxy list is empty or the last value cannot convert to u64
    pub fn last_insert_id(&self) -> u64 {
        match self {
            Self::Empty | Self::Conflicted => 0,
            Self::Inserted(val) => {
                let ret = val
                    .last()
                    .expect("Cannot get last value of proxy insert result");
                let ret: u64 = ret.clone().into();
                ret
            }
        }
    }

    /// Get the number of rows affected by the operation
    pub fn rows_affected(&self) -> u64 {
        match self {
            Self::Empty | Self::Conflicted => 0,
            Self::Inserted(val) => val.len() as u64,
        }
    }
}

impl Default for ExecResultHolder {
    fn default() -> Self {
        Self::Proxy(ProxyExecResult::default())
    }
}

impl From<ProxyExecResult> for ExecResult {
    fn from(result: ProxyExecResult) -> Self {
        Self {
            result: ExecResultHolder::Proxy(result),
        }
    }
}

impl From<ExecResult> for ProxyExecResult {
    fn from(result: ExecResult) -> Self {
        match result.result {
            #[cfg(feature = "sqlx-mysql")]
            ExecResultHolder::SqlxMySql(result) => Self {
                last_insert_id: ProxyInsertResult::Inserted(vec![json!(
                    result.last_insert_id() as u64
                )]),
                rows_affected: result.rows_affected(),
            },
            #[cfg(feature = "sqlx-postgres")]
            ExecResultHolder::SqlxPostgres(result) => Self {
                last_insert_id: ProxyInsertResult::Empty,
                rows_affected: result.rows_affected(),
            },
            #[cfg(feature = "sqlx-sqlite")]
            ExecResultHolder::SqlxSqlite(result) => Self {
                last_insert_id: ProxyInsertResult::Inserted(vec![json!(
                    result.last_insert_rowid() as u64
                )]),
                rows_affected: result.rows_affected(),
            },
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => {
                ProxyExecResult::Inserted(vec![ProxyExecResultIdType::Integer(
                    result.last_insert_id,
                )])
            }
            ExecResultHolder::Proxy(result) => result,
        }
    }
}

/// Defines the structure of a Row for the [ProxyDatabase]
/// which is just a [BTreeMap]<[String], [Value]>
#[derive(Clone, Debug)]
pub struct ProxyRow {
    /// The values of the single row
    pub values: BTreeMap<String, Value>,
}

impl ProxyRow {
    /// Create a new [ProxyRow] from a [BTreeMap]<[String], [Value]>
    pub fn new(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }
}

impl Default for ProxyRow {
    fn default() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }
}

impl From<BTreeMap<String, Value>> for ProxyRow {
    fn from(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }
}

impl From<ProxyRow> for BTreeMap<String, Value> {
    fn from(row: ProxyRow) -> Self {
        row.values
    }
}

impl From<ProxyRow> for Vec<(String, Value)> {
    fn from(row: ProxyRow) -> Self {
        row.values.into_iter().collect()
    }
}

impl From<ProxyRow> for QueryResult {
    fn from(row: ProxyRow) -> Self {
        QueryResult {
            row: QueryResultRow::Proxy(row),
        }
    }
}

#[cfg(feature = "with-json")]
impl Into<serde_json::Value> for ProxyRow {
    fn into(self) -> serde_json::Value {
        self.values
            .into_iter()
            .map(|(k, v)| (k, sea_query::sea_value_to_json_value(&v)))
            .collect()
    }
}

/// Convert [QueryResult] to [ProxyRow]
pub fn from_query_result_to_proxy_row(result: &QueryResult) -> ProxyRow {
    match &result.row {
        #[cfg(feature = "sqlx-mysql")]
        QueryResultRow::SqlxMySql(row) => from_sqlx_mysql_row_to_proxy_row(&row),
        #[cfg(feature = "sqlx-postgres")]
        QueryResultRow::SqlxPostgres(row) => from_sqlx_postgres_row_to_proxy_row(&row),
        #[cfg(feature = "sqlx-sqlite")]
        QueryResultRow::SqlxSqlite(row) => from_sqlx_sqlite_row_to_proxy_row(&row),
        #[cfg(feature = "mock")]
        QueryResultRow::Mock(row) => ProxyRow {
            values: row.values.clone(),
        },
        QueryResultRow::Proxy(row) => row.to_owned(),
    }
}

#[cfg(feature = "sqlx-mysql")]
pub(crate) fn from_sqlx_mysql_row_to_proxy_row(row: &sqlx::mysql::MySqlRow) -> ProxyRow {
    // https://docs.rs/sqlx-mysql/0.7.2/src/sqlx_mysql/protocol/text/column.rs.html
    // https://docs.rs/sqlx-mysql/0.7.2/sqlx_mysql/types/index.html
    use sqlx::{Column, Row, TypeInfo};
    ProxyRow {
        values: row
            .columns()
            .iter()
            .map(|c| {
                (
                    c.name().to_string(),
                    match c.type_info().name() {
                        "TINYINT(1)" | "BOOLEAN" => Value::Bool(Some(
                            row.try_get(c.ordinal()).expect("Failed to get boolean"),
                        )),
                        "TINYINT UNSIGNED" => Value::TinyUnsigned(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get unsigned tiny integer"),
                        )),
                        "SMALLINT UNSIGNED" => Value::SmallUnsigned(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get unsigned small integer"),
                        )),
                        "INT UNSIGNED" => Value::Unsigned(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get unsigned integer"),
                        )),
                        "MEDIUMINT UNSIGNED" | "BIGINT UNSIGNED" => Value::BigUnsigned(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get unsigned big integer"),
                        )),
                        "TINYINT" => Value::TinyInt(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get tiny integer"),
                        )),
                        "SMALLINT" => Value::SmallInt(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get small integer"),
                        )),
                        "INT" => Value::Int(Some(
                            row.try_get(c.ordinal()).expect("Failed to get integer"),
                        )),
                        "MEDIUMINT" | "BIGINT" => Value::BigInt(Some(
                            row.try_get(c.ordinal()).expect("Failed to get big integer"),
                        )),
                        "FLOAT" => Value::Float(Some(
                            row.try_get(c.ordinal()).expect("Failed to get float"),
                        )),
                        "DOUBLE" => Value::Double(Some(
                            row.try_get(c.ordinal()).expect("Failed to get double"),
                        )),

                        "BIT" | "BINARY" | "VARBINARY" | "TINYBLOB" | "BLOB" | "MEDIUMBLOB"
                        | "LONGBLOB" => Value::Bytes(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get bytes"),
                        ))),

                        "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                            Value::String(Some(Box::new(
                                row.try_get(c.ordinal()).expect("Failed to get string"),
                            )))
                        }

                        #[cfg(feature = "with-chrono")]
                        "TIMESTAMP" => Value::ChronoDateTimeUtc(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamp"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMESTAMP" => Value::TimeDateTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamp"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "DATE" => Value::ChronoDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get date"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATE" => Value::TimeDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get date"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "TIME" => Value::ChronoTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get time"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIME" => Value::TimeTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get time"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "DATETIME" => Value::ChronoDateTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get datetime"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATETIME" => Value::TimeDateTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get datetime"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "YEAR" => Value::ChronoDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get year"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "YEAR" => Value::TimeDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get year"),
                        ))),

                        "ENUM" | "SET" | "GEOMETRY" => Value::String(Some(Box::new(
                            row.try_get(c.ordinal())
                                .expect("Failed to get serialized string"),
                        ))),

                        #[cfg(feature = "with-bigdecimal")]
                        "DECIMAL" => Value::BigDecimal(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get decimal"),
                        ))),
                        #[cfg(all(
                            feature = "with-rust_decimal",
                            not(feature = "with-bigdecimal")
                        ))]
                        "DECIMAL" => Value::Decimal(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get decimal"),
                        ))),

                        #[cfg(feature = "with-json")]
                        "JSON" => Value::Json(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get json"),
                        ))),

                        _ => unreachable!("Unknown column type: {}", c.type_info().name()),
                    },
                )
            })
            .collect(),
    }
}

#[cfg(feature = "sqlx-postgres")]
pub(crate) fn from_sqlx_postgres_row_to_proxy_row(row: &sqlx::postgres::PgRow) -> ProxyRow {
    // https://docs.rs/sqlx-postgres/0.7.2/src/sqlx_postgres/type_info.rs.html
    // https://docs.rs/sqlx-postgres/0.7.2/sqlx_postgres/types/index.html
    use sqlx::{Column, Row, TypeInfo};
    ProxyRow {
        values: row
            .columns()
            .iter()
            .map(|c| {
                (
                    c.name().to_string(),
                    match c.type_info().name() {
                        "BOOL" => Value::Bool(Some(
                            row.try_get(c.ordinal()).expect("Failed to get boolean"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "BOOL[]" => Value::Array(
                            sea_query::ArrayType::Bool,
                            Some(Box::new(
                                row.try_get::<Vec<bool>, _>(c.ordinal())
                                    .expect("Failed to get boolean array")
                                    .iter()
                                    .map(|val| Value::Bool(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "\"CHAR\"" => Value::TinyInt(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get small integer"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "\"CHAR\"[]" => Value::Array(
                            sea_query::ArrayType::TinyInt,
                            Some(Box::new(
                                row.try_get::<Vec<i8>, _>(c.ordinal())
                                    .expect("Failed to get small integer array")
                                    .iter()
                                    .map(|val| Value::TinyInt(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "SMALLINT" | "SMALLSERIAL" | "INT2" => Value::SmallInt(Some(
                            row.try_get(c.ordinal())
                                .expect("Failed to get small integer"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "SMALLINT[]" | "SMALLSERIAL[]" | "INT2[]" => Value::Array(
                            sea_query::ArrayType::SmallInt,
                            Some(Box::new(
                                row.try_get::<Vec<i16>, _>(c.ordinal())
                                    .expect("Failed to get small integer array")
                                    .iter()
                                    .map(|val| Value::SmallInt(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "INT" | "SERIAL" | "INT4" => Value::Int(Some(
                            row.try_get(c.ordinal()).expect("Failed to get integer"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "INT[]" | "SERIAL[]" | "INT4[]" => Value::Array(
                            sea_query::ArrayType::Int,
                            Some(Box::new(
                                row.try_get::<Vec<i32>, _>(c.ordinal())
                                    .expect("Failed to get integer array")
                                    .iter()
                                    .map(|val| Value::Int(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "BIGINT" | "BIGSERIAL" | "INT8" => Value::BigInt(Some(
                            row.try_get(c.ordinal()).expect("Failed to get big integer"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "BIGINT[]" | "BIGSERIAL[]" | "INT8[]" => Value::Array(
                            sea_query::ArrayType::BigInt,
                            Some(Box::new(
                                row.try_get::<Vec<i64>, _>(c.ordinal())
                                    .expect("Failed to get big integer array")
                                    .iter()
                                    .map(|val| Value::BigInt(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "FLOAT4" | "REAL" => Value::Float(Some(
                            row.try_get(c.ordinal()).expect("Failed to get float"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "FLOAT4[]" | "REAL[]" => Value::Array(
                            sea_query::ArrayType::Float,
                            Some(Box::new(
                                row.try_get::<Vec<f32>, _>(c.ordinal())
                                    .expect("Failed to get float array")
                                    .iter()
                                    .map(|val| Value::Float(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "FLOAT8" | "DOUBLE PRECISION" => Value::Double(Some(
                            row.try_get(c.ordinal()).expect("Failed to get double"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "FLOAT8[]" | "DOUBLE PRECISION[]" => Value::Array(
                            sea_query::ArrayType::Double,
                            Some(Box::new(
                                row.try_get::<Vec<f64>, _>(c.ordinal())
                                    .expect("Failed to get double array")
                                    .iter()
                                    .map(|val| Value::Double(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "VARCHAR" | "CHAR" | "TEXT" | "NAME" => Value::String(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get string"),
                        ))),
                        #[cfg(feature = "postgres-array")]
                        "VARCHAR[]" | "CHAR[]" | "TEXT[]" | "NAME[]" => Value::Array(
                            sea_query::ArrayType::String,
                            Some(Box::new(
                                row.try_get::<Vec<String>, _>(c.ordinal())
                                    .expect("Failed to get string array")
                                    .iter()
                                    .map(|val| Value::String(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        "BYTEA" => Value::Bytes(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get bytes"),
                        ))),
                        #[cfg(feature = "postgres-array")]
                        "BYTEA[]" => Value::Array(
                            sea_query::ArrayType::Bytes,
                            Some(Box::new(
                                row.try_get::<Vec<Vec<u8>>, _>(c.ordinal())
                                    .expect("Failed to get bytes array")
                                    .iter()
                                    .map(|val| Value::Bytes(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-bigdecimal")]
                        "NUMERIC" => Value::BigDecimal(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get numeric"),
                        ))),
                        #[cfg(all(
                            feature = "with-rust_decimal",
                            not(feature = "with-bigdecimal")
                        ))]
                        "NUMERIC" => Value::Decimal(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get numeric"),
                        ))),

                        #[cfg(all(feature = "with-bigdecimal", feature = "postgres-array"))]
                        "NUMERIC[]" => Value::Array(
                            sea_query::ArrayType::BigDecimal,
                            Some(Box::new(
                                row.try_get::<Vec<bigdecimal::BigDecimal>, _>(c.ordinal())
                                    .expect("Failed to get numeric array")
                                    .iter()
                                    .map(|val| Value::BigDecimal(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),
                        #[cfg(all(
                            feature = "with-rust_decimal",
                            not(feature = "with-bigdecimal"),
                            feature = "postgres-array"
                        ))]
                        "NUMERIC[]" => Value::Array(
                            sea_query::ArrayType::Decimal,
                            Some(Box::new(
                                row.try_get::<Vec<rust_decimal::Decimal>, _>(c.ordinal())
                                    .expect("Failed to get numeric array")
                                    .iter()
                                    .map(|val| Value::Decimal(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        "OID" => Value::BigInt(Some(
                            row.try_get(c.ordinal()).expect("Failed to get oid"),
                        )),
                        #[cfg(feature = "postgres-array")]
                        "OID[]" => Value::Array(
                            sea_query::ArrayType::BigInt,
                            Some(Box::new(
                                row.try_get::<Vec<i64>, _>(c.ordinal())
                                    .expect("Failed to get oid array")
                                    .iter()
                                    .map(|val| Value::BigInt(Some(*val)))
                                    .collect(),
                            )),
                        ),

                        "JSON" | "JSONB" => Value::Json(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get json"),
                        ))),
                        #[cfg(any(feature = "json-array", feature = "postgres-array"))]
                        "JSON[]" | "JSONB[]" => Value::Array(
                            sea_query::ArrayType::Json,
                            Some(Box::new(
                                row.try_get::<Vec<serde_json::Value>, _>(c.ordinal())
                                    .expect("Failed to get json array")
                                    .iter()
                                    .map(|val| Value::Json(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-ipnetwork")]
                        "INET" | "CIDR" => Value::IpNetwork(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get ip address"),
                        ))),
                        #[cfg(feature = "with-ipnetwork")]
                        "INET[]" | "CIDR[]" => Value::Array(
                            sea_query::ArrayType::IpNetwork,
                            Some(Box::new(
                                row.try_get::<Vec<ipnetwork::IpNetwork>, _>(c.ordinal())
                                    .expect("Failed to get ip address array")
                                    .iter()
                                    .map(|val| Value::IpNetwork(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-mac_address")]
                        "MACADDR" | "MACADDR8" => Value::MacAddress(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get mac address"),
                        ))),
                        #[cfg(all(feature = "with-mac_address", feature = "postgres-array"))]
                        "MACADDR[]" | "MACADDR8[]" => Value::Array(
                            sea_query::ArrayType::MacAddress,
                            Some(Box::new(
                                row.try_get::<Vec<mac_address::MacAddress>, _>(c.ordinal())
                                    .expect("Failed to get mac address array")
                                    .iter()
                                    .map(|val| Value::MacAddress(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIMESTAMP" => Value::ChronoDateTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamp"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMESTAMP" => Value::TimeDateTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamp"),
                        ))),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIMESTAMP[]" => Value::Array(
                            sea_query::ArrayType::ChronoDateTime,
                            Some(Box::new(
                                row.try_get::<Vec<chrono::NaiveDateTime>, _>(c.ordinal())
                                    .expect("Failed to get timestamp array")
                                    .iter()
                                    .map(|val| Value::ChronoDateTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIMESTAMP[]" => Value::Array(
                            sea_query::ArrayType::TimeDateTime,
                            Some(Box::new(
                                row.try_get::<Vec<time::OffsetDateTime>, _>(c.ordinal())
                                    .expect("Failed to get timestamp array")
                                    .iter()
                                    .map(|val| Value::TimeDateTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "DATE" => Value::ChronoDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get date"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATE" => Value::TimeDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get date"),
                        ))),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "DATE[]" => Value::Array(
                            sea_query::ArrayType::ChronoDate,
                            Some(Box::new(
                                row.try_get::<Vec<chrono::NaiveDate>, _>(c.ordinal())
                                    .expect("Failed to get date array")
                                    .iter()
                                    .map(|val| Value::ChronoDate(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "DATE[]" => Value::Array(
                            sea_query::ArrayType::TimeDate,
                            Some(Box::new(
                                row.try_get::<Vec<time::Date>, _>(c.ordinal())
                                    .expect("Failed to get date array")
                                    .iter()
                                    .map(|val| Value::TimeDate(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIME" => Value::ChronoTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get time"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIME" => Value::TimeTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get time"),
                        ))),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIME[]" => Value::Array(
                            sea_query::ArrayType::ChronoTime,
                            Some(Box::new(
                                row.try_get::<Vec<chrono::NaiveTime>, _>(c.ordinal())
                                    .expect("Failed to get time array")
                                    .iter()
                                    .map(|val| Value::ChronoTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIME[]" => Value::Array(
                            sea_query::ArrayType::TimeTime,
                            Some(Box::new(
                                row.try_get::<Vec<time::Time>, _>(c.ordinal())
                                    .expect("Failed to get time array")
                                    .iter()
                                    .map(|val| Value::TimeTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIMESTAMPTZ" => Value::ChronoDateTimeUtc(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamptz"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMESTAMPTZ" => Value::TimeDateTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamptz"),
                        ))),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIMESTAMPTZ[]" => Value::Array(
                            sea_query::ArrayType::ChronoDateTimeUtc,
                            Some(Box::new(
                                row.try_get::<Vec<chrono::DateTime<chrono::Utc>>, _>(c.ordinal())
                                    .expect("Failed to get timestamptz array")
                                    .iter()
                                    .map(|val| {
                                        Value::ChronoDateTimeUtc(Some(Box::new(val.clone())))
                                    })
                                    .collect(),
                            )),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIMESTAMPTZ[]" => Value::Array(
                            sea_query::ArrayType::TimeDateTime,
                            Some(Box::new(
                                row.try_get::<Vec<time::OffsetDateTime>, _>(c.ordinal())
                                    .expect("Failed to get timestamptz array")
                                    .iter()
                                    .map(|val| Value::TimeDateTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIMETZ" => Value::ChronoTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timetz"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMETZ" => Value::TimeTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timetz"),
                        ))),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIMETZ[]" => Value::Array(
                            sea_query::ArrayType::ChronoTime,
                            Some(Box::new(
                                row.try_get::<Vec<chrono::NaiveTime>, _>(c.ordinal())
                                    .expect("Failed to get timetz array")
                                    .iter()
                                    .map(|val| Value::ChronoTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIMETZ[]" => Value::Array(
                            sea_query::ArrayType::TimeTime,
                            Some(Box::new(
                                row.try_get::<Vec<time::Time>, _>(c.ordinal())
                                    .expect("Failed to get timetz array")
                                    .iter()
                                    .map(|val| Value::TimeTime(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        #[cfg(feature = "with-uuid")]
                        "UUID" => Value::Uuid(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get uuid"),
                        ))),

                        #[cfg(all(feature = "with-uuid", feature = "postgres-array"))]
                        "UUID[]" => Value::Array(
                            sea_query::ArrayType::Uuid,
                            Some(Box::new(
                                row.try_get::<Vec<uuid::Uuid>, _>(c.ordinal())
                                    .expect("Failed to get uuid array")
                                    .iter()
                                    .map(|val| Value::Uuid(Some(Box::new(val.clone()))))
                                    .collect(),
                            )),
                        ),

                        _ => unreachable!("Unknown column type: {}", c.type_info().name()),
                    },
                )
            })
            .collect(),
    }
}

#[cfg(feature = "sqlx-sqlite")]
pub(crate) fn from_sqlx_sqlite_row_to_proxy_row(row: &sqlx::sqlite::SqliteRow) -> ProxyRow {
    // https://docs.rs/sqlx-sqlite/0.7.2/src/sqlx_sqlite/type_info.rs.html
    // https://docs.rs/sqlx-sqlite/0.7.2/sqlx_sqlite/types/index.html
    use sqlx::{Column, Row, TypeInfo};
    ProxyRow {
        values: row
            .columns()
            .iter()
            .map(|c| {
                (
                    c.name().to_string(),
                    match c.type_info().name() {
                        "BOOLEAN" => Value::Bool(Some(
                            row.try_get(c.ordinal()).expect("Failed to get boolean"),
                        )),

                        "INTEGER" => Value::Int(Some(
                            row.try_get(c.ordinal()).expect("Failed to get integer"),
                        )),

                        "BIGINT" | "INT8" => Value::BigInt(Some(
                            row.try_get(c.ordinal()).expect("Failed to get big integer"),
                        )),

                        "REAL" => Value::Double(Some(
                            row.try_get(c.ordinal()).expect("Failed to get double"),
                        )),

                        "TEXT" => Value::String(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get string"),
                        ))),

                        "BLOB" => Value::Bytes(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get bytes"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "DATETIME" => Value::ChronoDateTimeUtc(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamp"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATETIME" => Value::TimeDateTimeWithTimeZone(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get timestamp"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "DATE" => Value::ChronoDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get date"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATE" => Value::TimeDate(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get date"),
                        ))),

                        #[cfg(feature = "with-chrono")]
                        "TIME" => Value::ChronoTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get time"),
                        ))),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIME" => Value::TimeTime(Some(Box::new(
                            row.try_get(c.ordinal()).expect("Failed to get time"),
                        ))),

                        _ => unreachable!("Unknown column type: {}", c.type_info().name()),
                    },
                )
            })
            .collect(),
    }
}

impl ProxyRow {
    /// Get a value from the [ProxyRow]
    pub fn try_get<T, I: crate::ColIdx>(&self, index: I) -> Result<T, DbErr>
    where
        T: ValueType,
    {
        if let Some(index) = index.as_str() {
            T::try_from(
                self.values
                    .get(index)
                    .ok_or_else(|| query_err(format!("No column for ColIdx {index:?}")))?
                    .clone(),
            )
            .map_err(type_err)
        } else if let Some(index) = index.as_usize() {
            let (_, value) = self
                .values
                .iter()
                .nth(*index)
                .ok_or_else(|| query_err(format!("Column at index {index} not found")))?;
            T::try_from(value.clone()).map_err(type_err)
        } else {
            unreachable!("Missing ColIdx implementation for ProxyRow");
        }
    }

    /// An iterator over the keys and values of a proxy row
    pub fn into_column_value_tuples(self) -> impl Iterator<Item = (String, Value)> {
        self.values.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entity::*, tests_cfg::*, Database, DbBackend, DbErr, ProxyDatabaseTrait, ProxyExecResult,
        ProxyExecResultIdType, ProxyRow, Statement,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct ProxyDb {}

    impl ProxyDatabaseTrait for ProxyDb {
        fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
            println!("SQL query: {}", statement.sql);
            Ok(vec![].into())
        }

        fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
            println!("SQL execute: {}", statement.sql);
            Ok(ProxyExecResult::Inserted(vec![
                ProxyExecResultIdType::Integer(1),
            ]))
        }
    }

    #[smol_potat::test]
    async fn create_proxy_conn() {
        let _db =
            Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
                .await
                .unwrap();
    }

    #[smol_potat::test]
    async fn select_rows() {
        let db =
            Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
                .await
                .unwrap();

        let _ = cake::Entity::find().all(&db).await;
    }

    #[smol_potat::test]
    async fn insert_one_row() {
        let db =
            Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
                .await
                .unwrap();

        let item = cake::ActiveModel {
            id: NotSet,
            name: Set("Alice".to_string()),
        };

        cake::Entity::insert(item).exec(&db).await.unwrap();
    }
}
