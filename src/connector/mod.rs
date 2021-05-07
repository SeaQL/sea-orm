mod select;

pub use select::*;

use crate::{DatabaseConnection, QueryResult};
use async_trait::async_trait;
use sea_query::{inject_parameters, MySqlQueryBuilder, Values};
use std::{error::Error, fmt};

pub struct Statement {
    pub sql: String,
    pub values: Values,
}

#[async_trait]
pub trait Connector {
    fn accepts(string: &str) -> bool;

    async fn connect(string: &str) -> Result<DatabaseConnection, ConnectionErr>;
}

#[async_trait]
pub trait Connection {
    async fn query_one(&self, stmt: Statement) -> Result<QueryResult, QueryErr>;

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, QueryErr>;
}

#[derive(Debug)]
pub struct QueryErr;

#[derive(Debug)]
pub struct ConnectionErr;

// Statement //

impl From<(String, Values)> for Statement {
    fn from(stmt: (String, Values)) -> Statement {
        Statement {
            sql: stmt.0,
            values: stmt.1,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = inject_parameters(
            &self.sql,
            self.values.0.clone(),
            &MySqlQueryBuilder::default(),
        );
        write!(f, "{}", &string)
    }
}

// QueryErr //

impl Error for QueryErr {}

impl fmt::Display for QueryErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ConnectionErr //

impl Error for ConnectionErr {}

impl fmt::Display for ConnectionErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
