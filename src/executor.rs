use crate::QueryResult;
use async_trait::async_trait;
use sea_query::Values;
use std::{error::Error, fmt};

pub struct Statement {
    pub sql: String,
    pub values: Values,
}

#[async_trait]
pub trait Executor {
    async fn query_one(&self, stmt: Statement) -> Result<QueryResult, ExecErr>;

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, ExecErr>;
}

#[derive(Debug)]
pub struct ExecErr;

// ----- //

impl From<(String, Values)> for Statement {
    fn from(stmt: (String, Values)) -> Statement {
        Statement {
            sql: stmt.0,
            values: stmt.1,
        }
    }
}

impl Error for ExecErr {}

impl fmt::Display for ExecErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
