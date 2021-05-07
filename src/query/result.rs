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

// QueryResult //

impl QueryResult {
    pub fn try_get_i32(&self, col: &str) -> Result<i32, TypeErr> {
        match &self.row {
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;

                if let Ok(val) = row.try_get(col) {
                    Ok(val)
                } else {
                    Err(TypeErr)
                }
            }
        }
    }

    pub fn try_get_string(&self, col: &str) -> Result<String, TypeErr> {
        match &self.row {
            QueryResultRow::SqlxMySql(row) => {
                use sqlx::Row;

                if let Ok(val) = row.try_get(col) {
                    Ok(val)
                } else {
                    Err(TypeErr)
                }
            }
        }
    }
}

// TypeErr //

impl Error for TypeErr {}

impl fmt::Display for TypeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
