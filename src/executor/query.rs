use crate::DbErr;
use chrono::NaiveDateTime;
use std::fmt;

#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

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

pub trait TryGetable {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr>
    where
        Self: Sized;
}

// QueryResult //

impl QueryResult {
    pub fn try_get<T>(&self, pre: &str, col: &str) -> Result<T, DbErr>
    where
        T: TryGetable,
    {
        T::try_get(self, pre, col)
    }
}

impl fmt::Debug for QueryResultRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            Self::SqlxMySql(row) => write!(f, "{:?}", row),
            #[cfg(feature = "sqlx-postgres")]
            Self::SqlxPostgres(_) => panic!("QueryResultRow::SqlxPostgres cannot be inspected"),
            #[cfg(feature = "sqlx-sqlite")]
            Self::SqlxSqlite(_) => panic!("QueryResultRow::SqlxSqlite cannot be inspected"),
            #[cfg(feature = "mock")]
            Self::Mock(row) => write!(f, "{:?}", row),
        }
    }
}

// TryGetable //

macro_rules! try_getable_all {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get(column.as_str())
                            .map_err(crate::sqlx_error_to_query_err)
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        row.try_get(column.as_str())
                            .map_err(crate::sqlx_error_to_query_err)
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get(column.as_str())
                            .map_err(crate::sqlx_error_to_query_err)
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => Ok(row.try_get(column.as_str())?),
                }
            }
        }

        impl TryGetable for Option<$type> {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => match row.try_get(column.as_str()) {
                        Ok(v) => Ok(Some(v)),
                        Err(_) => Ok(None),
                    },
                }
            }
        }
    };
}

macro_rules! try_getable_unsigned {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get(column.as_str())
                            .map_err(crate::sqlx_error_to_query_err)
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-postgres", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        row.try_get(column.as_str())
                            .map_err(crate::sqlx_error_to_query_err)
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => Ok(row.try_get(column.as_str())?),
                }
            }
        }

        impl TryGetable for Option<$type> {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-postgres", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => match row.try_get(column.as_str()) {
                        Ok(v) => Ok(Some(v)),
                        Err(_) => Ok(None),
                    },
                }
            }
        }
    };
}

macro_rules! try_getable_mysql {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        row.try_get(column.as_str())
                            .map_err(crate::sqlx_error_to_query_err)
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
                    QueryResultRow::Mock(row) => Ok(row.try_get(column.as_str())?),
                }
            }
        }

        impl TryGetable for Option<$type> {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    QueryResultRow::SqlxPostgres(_) => {
                        panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(_) => {
                        panic!("{} unsupported by sqlx-sqlite", stringify!($type))
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => match row.try_get(column.as_str()) {
                        Ok(v) => Ok(Some(v)),
                        Err(_) => Ok(None),
                    },
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
try_getable_all!(u32);
try_getable_mysql!(u64);
try_getable_all!(f32);
try_getable_all!(f64);
try_getable_all!(String);
try_getable_all!(NaiveDateTime);

#[cfg(feature = "with-rust_decimal")]
use rust_decimal::Decimal;

#[cfg(feature = "with-rust_decimal")]
impl TryGetable for Decimal {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
        let column = format!("{}{}", pre, col);
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                row.try_get(column.as_str())
                    .map_err(crate::sqlx_error_to_query_err)
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                row.try_get(column.as_str())
                    .map_err(crate::sqlx_error_to_query_err)
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => {
                use sqlx::Row;
                let val: f64 = row
                    .try_get(column.as_str())
                    .map_err(crate::sqlx_error_to_query_err)?;
                use rust_decimal::prelude::FromPrimitive;
                Decimal::from_f64(val)
                    .ok_or_else(|| DbErr::Query("Failed to convert f64 into Decimal".to_owned()))
            }
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => Ok(row.try_get(column.as_str())?),
        }
    }
}

#[cfg(feature = "with-rust_decimal")]
impl TryGetable for Option<Decimal> {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, DbErr> {
        let column = format!("{}{}", pre, col);
        match &res.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                match row.try_get(column.as_str()) {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Ok(None),
                }
            }
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => {
                use sqlx::Row;
                match row.try_get(column.as_str()) {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Ok(None),
                }
            }
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(_) => {
                let result: Result<Decimal, _> = TryGetable::try_get(res, pre, col);
                match result {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Ok(None),
                }
            }
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => match row.try_get(column.as_str()) {
                Ok(v) => Ok(Some(v)),
                Err(_) => Ok(None),
            },
        }
    }
}
