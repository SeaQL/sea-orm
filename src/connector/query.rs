use sqlx::mysql::MySqlRow;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct QueryResult {
    pub(crate) row: QueryResultRow,
}

#[derive(Debug)]
pub(crate) enum QueryResultRow {
    SqlxMySql(MySqlRow),
}

#[derive(Debug)]
pub struct TypeErr;

pub trait TryGetable {
    fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

// TryGetable //

macro_rules! try_getable {
    ( $type: ty ) => {
        impl TryGetable for $type {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        Ok(row.try_get(column.as_str())?)
                    }
                }
            }
        }

        impl TryGetable for Option<$type> {
            fn try_get(res: &QueryResult, pre: &str, col: &str) -> Result<Self, TypeErr> {
                let column = format!("{}{}", pre, col);
                match &res.row {
                    QueryResultRow::SqlxMySql(row) => {
                        use sqlx::Row;
                        match row.try_get(column.as_str()) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    }
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

// QueryResult //

impl QueryResult {
    pub fn try_get<T>(&self, pre: &str, col: &str) -> Result<T, TypeErr>
    where
        T: TryGetable,
    {
        T::try_get(self, pre, col)
    }
}

// TypeErr //

impl Error for TypeErr {}

impl fmt::Display for TypeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<sqlx::Error> for TypeErr {
    fn from(_: sqlx::Error) -> TypeErr {
        TypeErr
    }
}
