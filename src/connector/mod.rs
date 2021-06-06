mod executor;
mod insert;
mod select;
mod update;

pub use executor::*;
pub use insert::*;
pub use select::*;
pub use update::*;

use crate::{DatabaseConnection, QueryResult, Statement, TypeErr};
use async_trait::async_trait;
use std::{error::Error, fmt};

#[async_trait]
pub trait Connector {
    fn accepts(string: &str) -> bool;

    async fn connect(string: &str) -> Result<DatabaseConnection, ConnectionErr>;
}

#[async_trait]
pub trait Connection {
    async fn execute(&self, stmt: Statement) -> Result<ExecResult, ExecErr>;

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, QueryErr>;

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, QueryErr>;
}

#[derive(Debug)]
pub struct QueryErr;

#[derive(Debug)]
pub struct ConnectionErr;

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

// ConnectionErr //

impl Error for ConnectionErr {}

impl fmt::Display for ConnectionErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
