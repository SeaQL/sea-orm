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
    fn try_get(res: &QueryResult, col: &str) -> Result<Self, TypeErr>
    where
        Self: std::marker::Sized;
}

// TryGetable //

impl TryGetable for i32 {
    fn try_get(res: &QueryResult, col: &str) -> Result<Self, TypeErr> {
        match &res.row {
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                Ok(row.try_get(col)?)
            }
        }
    }
}

impl TryGetable for String {
    fn try_get(res: &QueryResult, col: &str) -> Result<Self, TypeErr> {
        match &res.row {
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;
                Ok(row.try_get(col)?)
            }
        }
    }
}

// QueryResult //

impl QueryResult {
    pub fn try_get<T>(&self, col: &str) -> Result<T, TypeErr>
    where
        T: TryGetable,
    {
        T::try_get(self, col)
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
