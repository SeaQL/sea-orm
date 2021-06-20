use std::{error::Error, fmt};

#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

pub(crate) enum QueryResultRow {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySql(sqlx::mysql::MySqlRow),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlite(sqlx::sqlite::SqliteRow),
    #[cfg(feature = "mock")]
    Mock(crate::MockRow),
}

#[derive(Debug)]
pub struct QueryErr;

#[derive(Debug)]
pub struct TypeErr;

pub trait TryGetable {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

// QueryResult //

impl QueryResult {
    pub fn try_get<T>(&self, pre: &str, col: &str) -> Result<T, TypeErr>
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
            #[cfg(feature = "sqlx-sqlite")]
            Self::SqlxSqlite(_) => panic!("QueryResultRow::SqlxSqlite cannot be inspected"),
            #[cfg(feature = "mock")]
            Self::Mock(row) => write!(f, "{:?}", row),
        }
    }
}

// QueryErr //

impl Error for QueryErr {}

impl fmt::Display for QueryErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TypeErr> for QueryErr {
    fn from(_: TypeErr) -> QueryErr {
        QueryErr
    }
}

// TypeErr //

impl Error for TypeErr {}

impl fmt::Display for TypeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TryGetable //

macro_rules! try_getable_all {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        Ok(row.try_get(column.as_str())?)
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(row) => {
                        use sqlx::Row;
                        Ok(row.try_get(column.as_str())?)
                    }
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => Ok(row.try_get(column.as_str())?),
                }
            }
        }

        impl TryGetable for Option<$type> {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr> {
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
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    #[cfg(feature = "sqlx-mysql")]
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        Ok(row.try_get(column.as_str())?)
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(_) => panic!("{} unsupported by sqlx-sqlite", stringify!($type)),
                    #[cfg(feature = "mock")]
                    QueryResultRow::Mock(row) => Ok(row.try_get(column.as_str())?),
                }
            }
        }

        impl TryGetable for Option<$type> {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr> {
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
                    #[cfg(feature = "sqlx-sqlite")]
                    QueryResultRow::SqlxSqlite(_) => panic!("{} unsupported by sqlx-sqlite", stringify!($type)),
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
try_getable_all!(u8);
try_getable_all!(u16);
try_getable_all!(u32);
try_getable_mysql!(u64);
try_getable_all!(f32);
try_getable_all!(f64);
try_getable_all!(String);
