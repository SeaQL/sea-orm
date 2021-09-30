use crate::{DbBackend, Statement};
use sea_query::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    stmts: Vec<Statement>,
}

impl Transaction {
    pub fn from_sql_and_values<I>(db_backend: DbBackend, sql: &str, values: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Self::one(Statement::from_string_values_tuple(
            db_backend,
            (sql.to_string(), values.into_iter().collect()),
        ))
    }

    /// Create a Transaction with one statement
    pub fn one(stmt: Statement) -> Self {
        Self { stmts: vec![stmt] }
    }

    /// Create a Transaction with many statements
    pub fn many<I>(stmts: I) -> Self
    where
        I: IntoIterator<Item = Statement>,
    {
        Self {
            stmts: stmts.into_iter().collect(),
        }
    }

    /// Wrap each Statement as a single-statement Transaction
    pub fn wrap<I>(stmts: I) -> Vec<Self>
    where
        I: IntoIterator<Item = Statement>,
    {
        stmts.into_iter().map(Self::one).collect()
    }
}
