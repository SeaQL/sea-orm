use sqlx::mysql::MySqlQueryResult;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ExecResult {
    pub(crate) result: ExecResultHolder,
}

#[derive(Debug)]
pub(crate) enum ExecResultHolder {
    SqlxMySql(MySqlQueryResult),
}

#[derive(Debug)]
pub struct ExecErr;

// ExecResult //

impl ExecResult {
    pub fn last_insert_id(&self) -> u64 {
        match &self.result {
            ExecResultHolder::SqlxMySql(result) => {
                result.last_insert_id()
            }
        }
    }

    pub fn rows_affected(&self) -> u64 {
        match &self.result {
            ExecResultHolder::SqlxMySql(result) => {
                result.rows_affected()
            }
        }
    }
}

// ExecErr //

impl Error for ExecErr {}

impl fmt::Display for ExecErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<sqlx::Error> for ExecErr {
    fn from(_: sqlx::Error) -> ExecErr {
        ExecErr
    }
}
