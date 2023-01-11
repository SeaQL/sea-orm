#[cfg(feature = "mock")]
use crate::debug_print;
use crate::{DbErr, SelectGetableValue, SelectorRaw, Statement};
use std::fmt;

/// Defines the result of a query operation on a Model
#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

#[allow(clippy::enum_variant_names)]
pub(crate) enum QueryResultRow {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySql(sqlx::mysql::MySqlRow),
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgres(sqlx::postgres::PgRow),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlite(sqlx::sqlite::SqliteRow),
    #[cfg(feature = "mock")]
    Mock(crate::MockRow),
}

/// An interface to get a value from the query result
pub trait TryGetable: Sized {
    /// Get a value from the query result with an ColIdx
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError>;

    /// Get a value from the query result with prefixed column name
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TryGetError> {
        let index = format!("{}{}", pre, col);
        Self::try_get_by(res, index.as_str())
    }

    /// Get a value from the query result based on the order in the select expressions
    fn try_get_by_index(res: &QueryResult, index: usize) -> Result<Self, TryGetError> {
        Self::try_get_by(res, index)
    }
}

/// An error from trying to get a row from a Model
#[derive(Debug)]
pub enum TryGetError {
    /// A database error was encountered as defined in [crate::DbErr]
    DbErr(DbErr),
    /// A null value was encountered
    Null(String),
}

impl From<TryGetError> for DbErr {
    fn from(e: TryGetError) -> DbErr {
        match e {
            TryGetError::DbErr(e) => e,
            TryGetError::Null(s) => {
                DbErr::Type(format!("A null value was encountered while decoding {}", s))
            }
        }
    }
}

// QueryResult //

impl QueryResult {
    /// Get a value from the query result with an ColIdx
    pub fn try_get_by<T, I>(&self, index: I) -> Result<T, DbErr>
    where
        T: TryGetable,
        I: ColIdx,
    {
        Ok(T::try_get_by(self, index)?)
    }

    /// Get a value from the query result with prefixed column name
    pub fn try_get<T>(&self, pre: &str, col: &str) -> Result<T, DbErr>
    where
        T: TryGetable,
    {
        Ok(T::try_get(self, pre, col)?)
    }

    /// Get a value from the query result based on the order in the select expressions
    pub fn try_get_by_index<T>(&self, idx: usize) -> Result<T, DbErr>
    where
        T: TryGetable,
    {
        Ok(T::try_get_by_index(self, idx)?)
    }

    /// Get a tuple value from the query result with prefixed column name
    pub fn try_get_many<T>(&self, pre: &str, cols: &[String]) -> Result<T, DbErr>
    where
        T: TryGetableMany,
    {
        Ok(T::try_get_many(self, pre, cols)?)
    }

    /// Get a tuple value from the query result based on the order in the select expressions
    pub fn try_get_many_by_index<T>(&self) -> Result<T, DbErr>
    where
        T: TryGetableMany,
    {
        Ok(T::try_get_many_by_index(self)?)
    }
}

#[allow(unused_variables)]
impl fmt::Debug for QueryResultRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            Self::SqlxMySql(row) => write!(f, "{:?}", row),
            #[cfg(feature = "sqlx-postgres")]
            Self::SqlxPostgres(_) => write!(f, "QueryResultRow::SqlxPostgres cannot be inspected"),
            #[cfg(feature = "sqlx-sqlite")]
            Self::SqlxSqlite(_) => write!(f, "QueryResultRow::SqlxSqlite cannot be inspected"),
            #[cfg(feature = "mock")]
            Self::Mock(row) => write!(f, "{:?}", row),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

// TryGetable //

impl<T: TryGetable> TryGetable for Option<T> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        match T::try_get_by(res, index) {
            Ok(v) => Ok(Some(v)),
            Err(TryGetError::Null(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Column Index, used by [`TryGetable`]. Implemented for `&str` and `usize`
pub trait ColIdx: std::fmt::Debug + Copy {
    #[cfg(feature = "sqlx-mysql")]
    /// Type surrogate
    type SqlxMySqlIndex: sqlx::ColumnIndex<sqlx::mysql::MySqlRow>;
    #[cfg(feature = "sqlx-postgres")]
    /// Type surrogate
    type SqlxPostgresIndex: sqlx::ColumnIndex<sqlx::postgres::PgRow>;
    #[cfg(feature = "sqlx-sqlite")]
    /// Type surrogate
    type SqlxSqliteIndex: sqlx::ColumnIndex<sqlx::sqlite::SqliteRow>;

    #[cfg(feature = "sqlx-mysql")]
    /// Basically a no-op; only to satisfy trait bounds
    fn as_sqlx_mysql_index(&self) -> Self::SqlxMySqlIndex;
    #[cfg(feature = "sqlx-postgres")]
    /// Basically a no-op; only to satisfy trait bounds
    fn as_sqlx_postgres_index(&self) -> Self::SqlxPostgresIndex;
    #[cfg(feature = "sqlx-sqlite")]
    /// Basically a no-op; only to satisfy trait bounds
    fn as_sqlx_sqlite_index(&self) -> Self::SqlxSqliteIndex;

    /// Self must be `&str`, return `None` otherwise
    fn as_str(&self) -> Option<&str>;
    /// Self must be `usize`, return `None` otherwise
    fn as_usize(&self) -> Option<&usize>;
}

impl ColIdx for &str {
    #[cfg(feature = "sqlx-mysql")]
    type SqlxMySqlIndex = Self;
    #[cfg(feature = "sqlx-postgres")]
    type SqlxPostgresIndex = Self;
    #[cfg(feature = "sqlx-sqlite")]
    type SqlxSqliteIndex = Self;

    #[cfg(feature = "sqlx-mysql")]
    #[inline]
    fn as_sqlx_mysql_index(&self) -> Self::SqlxMySqlIndex {
        self
    }
    #[cfg(feature = "sqlx-postgres")]
    #[inline]
    fn as_sqlx_postgres_index(&self) -> Self::SqlxPostgresIndex {
        self
    }
    #[cfg(feature = "sqlx-sqlite")]
    #[inline]
    fn as_sqlx_sqlite_index(&self) -> Self::SqlxSqliteIndex {
        self
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        Some(self)
    }
    #[inline]
    fn as_usize(&self) -> Option<&usize> {
        None
    }
}

impl ColIdx for usize {
    #[cfg(feature = "sqlx-mysql")]
    type SqlxMySqlIndex = Self;
    #[cfg(feature = "sqlx-postgres")]
    type SqlxPostgresIndex = Self;
    #[cfg(feature = "sqlx-sqlite")]
    type SqlxSqliteIndex = Self;

    #[cfg(feature = "sqlx-mysql")]
    #[inline]
    fn as_sqlx_mysql_index(&self) -> Self::SqlxMySqlIndex {
        *self
    }
    #[cfg(feature = "sqlx-postgres")]
    #[inline]
    fn as_sqlx_postgres_index(&self) -> Self::SqlxPostgresIndex {
        *self
    }
    #[cfg(feature = "sqlx-sqlite")]
    #[inline]
    fn as_sqlx_sqlite_index(&self) -> Self::SqlxSqliteIndex {
        *self
    }

    #[inline]
    fn as_str(&self) -> Option<&str> {
        None
    }
    #[inline]
    fn as_usize(&self) -> Option<&usize> {
        Some(self)
    }
}

macro_rules! try_getable_all {
    ( $type: ty ) => {
        impl TryGetable for $type {
            #[allow(unused_variables)]
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_mysql_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_postgres_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_sqlite_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        err_null_idx_col(idx)
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! try_getable_unsigned {
    ( $type: ty ) => {
        impl TryGetable for $type {
            #[allow(unused_variables)]
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_mysql_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-postgres", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_sqlite_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        err_null_idx_col(idx)
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! try_getable_mysql {
    ( $type: ty ) => {
        impl TryGetable for $type {
            #[allow(unused_variables)]
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_mysql_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-postgres", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(_) => {
                        panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        err_null_idx_col(idx)
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! try_getable_date_time {
    ( $type: ty ) => {
        impl TryGetable for $type {
            #[allow(unused_variables)]
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use chrono::{DateTime, Utc};
                        use sqlx::Row;
                        row.try_get::<Option<DateTime<Utc>>, _>(idx.as_sqlx_mysql_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                            .map(|v| v.into())
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<$type>, _>(idx.as_sqlx_postgres_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use chrono::{DateTime, Utc};
                        use sqlx::Row;
                        row.try_get::<Option<DateTime<Utc>>, _>(idx.as_sqlx_sqlite_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                            .map(|v| v.into())
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        err_null_idx_col(idx)
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    };
}

try_getable_all!(bool);
try_getable_all!(i8);
try_getable_all!(i16);
try_getable_all!(i32);
try_getable_all!(i64);
try_getable_unsigned!(u8);
try_getable_unsigned!(u16);
try_getable_mysql!(u64);
try_getable_all!(f32);
try_getable_all!(f64);
try_getable_all!(String);
try_getable_all!(Vec<u8>);

#[cfg(feature = "with-json")]
try_getable_all!(serde_json::Value);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::NaiveDate);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::NaiveTime);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::NaiveDateTime);

#[cfg(feature = "with-chrono")]
try_getable_date_time!(chrono::DateTime<chrono::FixedOffset>);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::DateTime<chrono::Utc>);

#[cfg(feature = "with-chrono")]
try_getable_all!(chrono::DateTime<chrono::Local>);

#[cfg(feature = "with-time")]
try_getable_all!(time::Date);

#[cfg(feature = "with-time")]
try_getable_all!(time::Time);

#[cfg(feature = "with-time")]
try_getable_all!(time::PrimitiveDateTime);

#[cfg(feature = "with-time")]
try_getable_all!(time::OffsetDateTime);

#[cfg(feature = "with-rust_decimal")]
use rust_decimal::Decimal;

#[cfg(feature = "with-rust_decimal")]
impl TryGetable for Decimal {
    #[allow(unused_variables)]
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<Decimal>, _>(idx.as_sqlx_mysql_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get::<Option<Decimal>, _>(idx.as_sqlx_postgres_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                let val: Option<f64> = row
                    .try_get(idx.as_sqlx_sqlite_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))?;
                match val {
                    Some(v) => Decimal::try_from(v).map_err(|e| {
                        TryGetError::DbErr(DbErr::TryIntoErr {
                            from: "f64",
                            into: "Decimal",
                            source: Box::new(e),
                        })
                    }),
                    None => Err(err_null_idx_col(idx)),
                }
            }
            #[cfg(feature = "mock")]
            #[allow(unused_variables)]
            QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                debug_print!("{:#?}", e.to_string());
                err_null_idx_col(idx)
            }),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "with-bigdecimal")]
use bigdecimal::BigDecimal;

#[cfg(feature = "with-bigdecimal")]
impl TryGetable for BigDecimal {
    #[allow(unused_variables)]
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<BigDecimal>, _>(idx.as_sqlx_mysql_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get::<Option<BigDecimal>, _>(idx.as_sqlx_postgres_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                let val: Option<f64> = row
                    .try_get(idx.as_sqlx_sqlite_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))?;
                match val {
                    Some(v) => BigDecimal::try_from(v).map_err(|e| {
                        TryGetError::DbErr(DbErr::TryIntoErr {
                            from: "f64",
                            into: "BigDecimal",
                            source: Box::new(e),
                        })
                    }),
                    None => Err(err_null_idx_col(idx)),
                }
            }
            #[cfg(feature = "mock")]
            #[allow(unused_variables)]
            QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                debug_print!("{:#?}", e.to_string());
                err_null_idx_col(idx)
            }),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[allow(unused_macros)]
macro_rules! try_getable_uuid {
    ( $type: ty, $conversion_fn: expr ) => {
        #[allow(unused_variables, unreachable_code)]
        impl TryGetable for $type {
            fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                let res: Result<uuid::Uuid, TryGetError> = match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<uuid::Uuid>, _>(idx.as_sqlx_mysql_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<uuid::Uuid>, _>(idx.as_sqlx_postgres_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get::<Option<uuid::Uuid>, _>(idx.as_sqlx_sqlite_index())
                            .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                            .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    }
                    #[cfg(feature = "mock")]
                    #[allow(unused_variables)]
                    QueryResultRow::Mock(row) => row.try_get::<uuid::Uuid, _>(idx).map_err(|e| {
                        debug_print!("{:#?}", e.to_string());
                        err_null_idx_col(idx)
                    }),
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                };
                res.map($conversion_fn)
            }
        }
    };
}

#[cfg(feature = "with-uuid")]
try_getable_uuid!(uuid::Uuid, Into::into);

#[cfg(feature = "with-uuid")]
try_getable_uuid!(uuid::fmt::Braced, uuid::Uuid::braced);

#[cfg(feature = "with-uuid")]
try_getable_uuid!(uuid::fmt::Hyphenated, uuid::Uuid::hyphenated);

#[cfg(feature = "with-uuid")]
try_getable_uuid!(uuid::fmt::Simple, uuid::Uuid::simple);

#[cfg(feature = "with-uuid")]
try_getable_uuid!(uuid::fmt::Urn, uuid::Uuid::urn);

impl TryGetable for u32 {
    #[allow(unused_variables)]
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<u32>, _>(idx.as_sqlx_mysql_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::postgres::types::Oid;
                // Since 0.6.0, SQLx has dropped direct mapping from PostgreSQL's OID to Rust's `u32`;
                // Instead, `u32` was wrapped by a `sqlx::Oid`.
                use sqlx::Row;
                row.try_get::<Option<Oid>, _>(idx.as_sqlx_postgres_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                    .map(|oid| oid.0)
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                row.try_get::<Option<u32>, _>(idx.as_sqlx_sqlite_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
            }
            #[cfg(feature = "mock")]
            #[allow(unused_variables)]
            QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                debug_print!("{:#?}", e.to_string());
                err_null_idx_col(idx)
            }),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[allow(dead_code)]
fn err_null_idx_col<I: ColIdx>(idx: I) -> TryGetError {
    TryGetError::Null(format!("{:?}", idx))
}

#[cfg(feature = "postgres-array")]
mod postgres_array {
    use super::*;

    #[allow(unused_macros)]
    macro_rules! try_getable_postgres_array {
        ( $type: ty ) => {
            #[allow(unused_variables)]
            impl TryGetable for Vec<$type> {
                fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                    match &res.row {
                        #[cfg(feature = "sqlx-mysql")]
                        QueryResultRow::SqlxMySql(_) => {
                            panic!("{} unsupported by sqlx-mysql", stringify!($type))
                        }
                        #[cfg(feature = "sqlx-postgres")]
                        QueryResultRow::SqlxPostgres(row) => {
                            use sqlx::Row;
                            row.try_get::<Option<Vec<$type>>, _>(idx.as_sqlx_postgres_index())
                                .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                                .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                        }
                        #[cfg(feature = "sqlx-sqlite")]
                        QueryResultRow::SqlxSqlite(_) => {
                            panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                        }
                        #[cfg(feature = "mock")]
                        #[allow(unused_variables)]
                        QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                            debug_print!("{:#?}", e.to_string());
                            err_null_idx_col(idx)
                        }),
                        #[allow(unreachable_patterns)]
                        _ => unreachable!(),
                    }
                }
            }
        };
    }

    try_getable_postgres_array!(bool);
    try_getable_postgres_array!(i8);
    try_getable_postgres_array!(i16);
    try_getable_postgres_array!(i32);
    try_getable_postgres_array!(i64);
    try_getable_postgres_array!(f32);
    try_getable_postgres_array!(f64);
    try_getable_postgres_array!(String);

    #[cfg(feature = "with-json")]
    try_getable_postgres_array!(serde_json::Value);

    #[cfg(feature = "with-chrono")]
    try_getable_postgres_array!(chrono::NaiveDate);

    #[cfg(feature = "with-chrono")]
    try_getable_postgres_array!(chrono::NaiveTime);

    #[cfg(feature = "with-chrono")]
    try_getable_postgres_array!(chrono::NaiveDateTime);

    #[cfg(feature = "with-chrono")]
    try_getable_postgres_array!(chrono::DateTime<chrono::FixedOffset>);

    #[cfg(feature = "with-chrono")]
    try_getable_postgres_array!(chrono::DateTime<chrono::Utc>);

    #[cfg(feature = "with-chrono")]
    try_getable_postgres_array!(chrono::DateTime<chrono::Local>);

    #[cfg(feature = "with-time")]
    try_getable_postgres_array!(time::Date);

    #[cfg(feature = "with-time")]
    try_getable_postgres_array!(time::Time);

    #[cfg(feature = "with-time")]
    try_getable_postgres_array!(time::PrimitiveDateTime);

    #[cfg(feature = "with-time")]
    try_getable_postgres_array!(time::OffsetDateTime);

    #[cfg(feature = "with-rust_decimal")]
    try_getable_postgres_array!(rust_decimal::Decimal);

    #[cfg(feature = "with-bigdecimal")]
    try_getable_postgres_array!(bigdecimal::BigDecimal);

    #[allow(unused_macros)]
    macro_rules! try_getable_postgres_array_uuid {
        ( $type: ty, $conversion_fn: expr ) => {
            #[allow(unused_variables, unreachable_code)]
            impl TryGetable for Vec<$type> {
                fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
                    let res: Result<Vec<uuid::Uuid>, TryGetError> = match &res.row {
                        #[cfg(feature = "sqlx-mysql")]
                        QueryResultRow::SqlxMySql(row) => {
                            panic!("{} unsupported by sqlx-mysql", stringify!($type))
                        }
                        #[cfg(feature = "sqlx-postgres")]
                        QueryResultRow::SqlxPostgres(row) => {
                            use sqlx::Row;
                            row.try_get::<Option<Vec<uuid::Uuid>>, _>(idx.as_sqlx_postgres_index())
                                .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                                .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                        }
                        #[cfg(feature = "sqlx-sqlite")]
                        QueryResultRow::SqlxSqlite(_) => {
                            panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                        }
                        #[cfg(feature = "mock")]
                        QueryResultRow::Mock(row) => {
                            row.try_get::<Vec<uuid::Uuid>, _>(idx).map_err(|e| {
                                debug_print!("{:#?}", e.to_string());
                                err_null_idx_col(idx)
                            })
                        }
                        #[allow(unreachable_patterns)]
                        _ => unreachable!(),
                    };
                    res.map(|vec| vec.into_iter().map($conversion_fn).collect())
                }
            }
        };
    }

    #[cfg(feature = "with-uuid")]
    try_getable_postgres_array_uuid!(uuid::Uuid, Into::into);

    #[cfg(feature = "with-uuid")]
    try_getable_postgres_array_uuid!(uuid::fmt::Braced, uuid::Uuid::braced);

    #[cfg(feature = "with-uuid")]
    try_getable_postgres_array_uuid!(uuid::fmt::Hyphenated, uuid::Uuid::hyphenated);

    #[cfg(feature = "with-uuid")]
    try_getable_postgres_array_uuid!(uuid::fmt::Simple, uuid::Uuid::simple);

    #[cfg(feature = "with-uuid")]
    try_getable_postgres_array_uuid!(uuid::fmt::Urn, uuid::Uuid::urn);

    impl TryGetable for Vec<u32> {
        #[allow(unused_variables)]
        fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
            match &res.row {
                #[cfg(feature = "sqlx-mysql")]
                QueryResultRow::SqlxMySql(_) => {
                    panic!("{} unsupported by sqlx-mysql", stringify!($type))
                }
                #[cfg(feature = "sqlx-postgres")]
                QueryResultRow::SqlxPostgres(row) => {
                    use sqlx::postgres::types::Oid;
                    // Since 0.6.0, SQLx has dropped direct mapping from PostgreSQL's OID to Rust's `u32`;
                    // Instead, `u32` was wrapped by a `sqlx::Oid`.
                    use sqlx::Row;
                    row.try_get::<Option<Vec<Oid>>, _>(idx.as_sqlx_postgres_index())
                        .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                        .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)))
                        .map(|oids| oids.into_iter().map(|oid| oid.0).collect())
                }
                #[cfg(feature = "sqlx-sqlite")]
                QueryResultRow::SqlxSqlite(_) => {
                    panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                }
                #[cfg(feature = "mock")]
                #[allow(unused_variables)]
                QueryResultRow::Mock(row) => row.try_get(idx).map_err(|e| {
                    debug_print!("{:#?}", e.to_string());
                    err_null_idx_col(idx)
                }),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }
}

// TryGetableMany //

/// An interface to get a tuple value from the query result
pub trait TryGetableMany: Sized {
    /// Get a tuple value from the query result with prefixed column name
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError>;

    /// Get a tuple value from the query result based on the order in the select expressions
    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError>;

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(all(feature = "mock", feature = "macros"))]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([[
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("New York Cheese"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DeriveIden, EnumIter, TryGetableMany};
    ///
    /// #[derive(EnumIter, DeriveIden)]
    /// enum ResultCol {
    ///     Name,
    ///     NumOfCakes,
    /// }
    ///
    /// let res: Vec<(String, i32)> =
    ///     <(String, i32)>::find_by_statement::<ResultCol>(Statement::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name", count("cake"."id") AS "num_of_cakes" FROM "cake""#,
    ///         [],
    ///     ))
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     [
    ///         ("Chocolate Forest".to_owned(), 1),
    ///         ("New York Cheese".to_owned(), 1),
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name", count("cake"."id") AS "num_of_cakes" FROM "cake""#,
    ///         []
    ///     ),]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn find_by_statement<C>(stmt: Statement) -> SelectorRaw<SelectGetableValue<Self, C>>
    where
        C: strum::IntoEnumIterator + sea_query::Iden,
    {
        SelectorRaw::<SelectGetableValue<Self, C>>::with_columns(stmt)
    }
}

impl<T> TryGetableMany for T
where
    T: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(1, cols)?;
        T::try_get(res, pre, &cols[0])
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        T::try_get_by_index(res, 0)
    }
}

impl<T> TryGetableMany for (T,)
where
    T: TryGetableMany,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        T::try_get_many(res, pre, cols).map(|r| (r,))
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        T::try_get_many_by_index(res).map(|r| (r,))
    }
}

impl<A, B> TryGetableMany for (A, B)
where
    A: TryGetable,
    B: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(2, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
        ))
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        Ok((A::try_get_by_index(res, 0)?, B::try_get_by_index(res, 1)?))
    }
}

impl<A, B, C> TryGetableMany for (A, B, C)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(3, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
        ))
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        Ok((
            A::try_get_by_index(res, 0)?,
            B::try_get_by_index(res, 1)?,
            C::try_get_by_index(res, 2)?,
        ))
    }
}

impl<A, B, C, D> TryGetableMany for (A, B, C, D)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
    D: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(4, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
            D::try_get(res, pre, &cols[3])?,
        ))
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        Ok((
            A::try_get_by_index(res, 0)?,
            B::try_get_by_index(res, 1)?,
            C::try_get_by_index(res, 2)?,
            D::try_get_by_index(res, 3)?,
        ))
    }
}

impl<A, B, C, D, E> TryGetableMany for (A, B, C, D, E)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
    D: TryGetable,
    E: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(5, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
            D::try_get(res, pre, &cols[3])?,
            E::try_get(res, pre, &cols[4])?,
        ))
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        Ok((
            A::try_get_by_index(res, 0)?,
            B::try_get_by_index(res, 1)?,
            C::try_get_by_index(res, 2)?,
            D::try_get_by_index(res, 3)?,
            E::try_get_by_index(res, 4)?,
        ))
    }
}

impl<A, B, C, D, E, F> TryGetableMany for (A, B, C, D, E, F)
where
    A: TryGetable,
    B: TryGetable,
    C: TryGetable,
    D: TryGetable,
    E: TryGetable,
    F: TryGetable,
{
    fn try_get_many(res: &QueryResult, pre: &str, cols: &[String]) -> Result<Self, TryGetError> {
        try_get_many_with_slice_len_of(6, cols)?;
        Ok((
            A::try_get(res, pre, &cols[0])?,
            B::try_get(res, pre, &cols[1])?,
            C::try_get(res, pre, &cols[2])?,
            D::try_get(res, pre, &cols[3])?,
            E::try_get(res, pre, &cols[4])?,
            F::try_get(res, pre, &cols[5])?,
        ))
    }

    fn try_get_many_by_index(res: &QueryResult) -> Result<Self, TryGetError> {
        Ok((
            A::try_get_by_index(res, 0)?,
            B::try_get_by_index(res, 1)?,
            C::try_get_by_index(res, 2)?,
            D::try_get_by_index(res, 3)?,
            E::try_get_by_index(res, 4)?,
            F::try_get_by_index(res, 5)?,
        ))
    }
}

fn try_get_many_with_slice_len_of(len: usize, cols: &[String]) -> Result<(), TryGetError> {
    if cols.len() < len {
        Err(TryGetError::DbErr(DbErr::Type(format!(
            "Expect {} column names supplied but got slice of length {}",
            len,
            cols.len()
        ))))
    } else {
        Ok(())
    }
}

// TryGetableFromJson //

/// An interface to get a JSON from the query result
#[cfg(feature = "with-json")]
pub trait TryGetableFromJson: Sized
where
    for<'de> Self: serde::Deserialize<'de>,
{
    /// Get a JSON from the query result with prefixed column name
    #[allow(unused_variables, unreachable_code)]
    fn try_get_from_json<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get::<Option<sqlx::types::Json<Self>>, _>(idx.as_sqlx_mysql_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)).map(|json| json.0))
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get::<Option<sqlx::types::Json<Self>>, _>(idx.as_sqlx_postgres_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)).map(|json| json.0))
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                row.try_get::<Option<sqlx::types::Json<Self>>, _>(idx.as_sqlx_sqlite_index())
                    .map_err(|e| TryGetError::DbErr(crate::sqlx_error_to_query_err(e)))
                    .and_then(|opt| opt.ok_or_else(|| err_null_idx_col(idx)).map(|json| json.0))
            }
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => row
                .try_get::<serde_json::Value, I>(idx)
                .map_err(|e| {
                    debug_print!("{:#?}", e.to_string());
                    err_null_idx_col(idx)
                })
                .and_then(|json| {
                    serde_json::from_value(json)
                        .map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
                }),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "with-json")]
impl<T> TryGetable for T
where
    T: TryGetableFromJson,
{
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        T::try_get_from_json(res, index)
    }
}

// TryFromU64 //
/// Try to convert a type to a u64
pub trait TryFromU64: Sized {
    /// The method to convert the type to a u64
    fn try_from_u64(n: u64) -> Result<Self, DbErr>;
}

macro_rules! try_from_u64_err {
    ( $type: ty ) => {
        impl TryFromU64 for $type {
            fn try_from_u64(_: u64) -> Result<Self, DbErr> {
                Err(DbErr::ConvertFromU64(stringify!($type)))
            }
        }
    };

    ( $($gen_type: ident),* ) => {
        impl<$( $gen_type, )*> TryFromU64 for ($( $gen_type, )*)
        where
            $( $gen_type: TryFromU64, )*
        {
            fn try_from_u64(_: u64) -> Result<Self, DbErr> {
                Err(DbErr::ConvertFromU64(stringify!($($gen_type,)*)))
            }
        }
    };
}

// impl TryFromU64 for tuples with generic types
try_from_u64_err!(A, B);
try_from_u64_err!(A, B, C);
try_from_u64_err!(A, B, C, D);
try_from_u64_err!(A, B, C, D, E);
try_from_u64_err!(A, B, C, D, E, F);

macro_rules! try_from_u64_numeric {
    ( $type: ty ) => {
        impl TryFromU64 for $type {
            fn try_from_u64(n: u64) -> Result<Self, DbErr> {
                use std::convert::TryInto;
                n.try_into().map_err(|e| DbErr::TryIntoErr {
                    from: stringify!(u64),
                    into: stringify!($type),
                    source: Box::new(e),
                })
            }
        }
    };
}

try_from_u64_numeric!(i8);
try_from_u64_numeric!(i16);
try_from_u64_numeric!(i32);
try_from_u64_numeric!(i64);
try_from_u64_numeric!(u8);
try_from_u64_numeric!(u16);
try_from_u64_numeric!(u32);
try_from_u64_numeric!(u64);

macro_rules! try_from_u64_string {
    ( $type: ty ) => {
        impl TryFromU64 for $type {
            fn try_from_u64(n: u64) -> Result<Self, DbErr> {
                Ok(n.to_string())
            }
        }
    };
}

try_from_u64_string!(String);

try_from_u64_err!(bool);
try_from_u64_err!(f32);
try_from_u64_err!(f64);
try_from_u64_err!(Vec<u8>);

#[cfg(feature = "with-json")]
try_from_u64_err!(serde_json::Value);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::NaiveDate);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::NaiveTime);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::NaiveDateTime);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::DateTime<chrono::FixedOffset>);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::DateTime<chrono::Utc>);

#[cfg(feature = "with-chrono")]
try_from_u64_err!(chrono::DateTime<chrono::Local>);

#[cfg(feature = "with-time")]
try_from_u64_err!(time::Date);

#[cfg(feature = "with-time")]
try_from_u64_err!(time::Time);

#[cfg(feature = "with-time")]
try_from_u64_err!(time::PrimitiveDateTime);

#[cfg(feature = "with-time")]
try_from_u64_err!(time::OffsetDateTime);

#[cfg(feature = "with-rust_decimal")]
try_from_u64_err!(rust_decimal::Decimal);

#[cfg(feature = "with-uuid")]
try_from_u64_err!(uuid::Uuid);

#[cfg(test)]
mod tests {
    use super::TryGetError;
    use crate::error::*;

    #[test]
    fn from_try_get_error() {
        // TryGetError::DbErr
        let try_get_error = TryGetError::DbErr(DbErr::Query(RuntimeErr::Internal(
            "expected error message".to_owned(),
        )));
        assert_eq!(
            DbErr::from(try_get_error),
            DbErr::Query(RuntimeErr::Internal("expected error message".to_owned()))
        );

        // TryGetError::Null
        let try_get_error = TryGetError::Null("column".to_owned());
        let expected = "A null value was encountered while decoding column".to_owned();
        assert_eq!(DbErr::from(try_get_error), DbErr::Type(expected));
    }
}
