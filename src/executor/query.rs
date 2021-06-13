use std::{error::Error, fmt};

#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

#[derive(Debug)]
pub(crate) enum QueryResultRow {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySql(sqlx::mysql::MySqlRow),
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

macro_rules! try_getable {
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

try_getable!(bool);
try_getable!(i8);
try_getable!(i16);
try_getable!(i32);
try_getable!(i64);
try_getable!(u8);
try_getable!(u16);
try_getable!(u32);
try_getable!(u64);
try_getable!(f32);
try_getable!(f64);
try_getable!(String);
