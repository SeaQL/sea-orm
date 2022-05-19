#[cfg(feature = "sqlx-error")]
use sqlx::Error;
#[cfg(feature = "sqlx-error")]
use std::fmt::{Display, Formatter};
#[cfg(feature = "sqlx-error")]
use std::sync::Arc;

/// An error from unsuccessful database operations
#[derive(Debug, PartialEq, Clone)]
pub enum DbErr {
    /// There was a problem with the database connection
    Conn(String),
    /// An operation did not execute successfully
    Exec(String),
    /// An error occurred while performing a query
    Query(String),
    /// The record was not found in the database
    RecordNotFound(String),
    /// A custom error
    Custom(String),
    /// Error occurred while parsing value as target type
    Type(String),
    /// Error occurred while parsing json value as target type
    Json(String),
    /// A migration error
    Migration(String),
    /// Error translated from Sqlx
    #[cfg(feature = "sqlx-error")]
    Sqlx(ErrFromSqlx),
}

/// A wrapper around the error, which might have been generated from sqlx
#[cfg(feature = "sqlx-error")]
#[derive(Debug, Clone)]
pub struct ErrFromSqlx {
    inner: Arc<sqlx::Error>,
    message: String,
}

#[cfg(feature = "sqlx-error")]
impl ErrFromSqlx {
    pub fn new(inner: sqlx::Error, message: String) -> Self {
        Self {
            inner: Arc::new(inner),
            message,
        }
    }

    pub fn inner(&self) -> &sqlx::Error {
        &self.inner
    }
}

#[cfg(feature = "sqlx-error")]
impl From<sqlx::Error> for ErrFromSqlx {
    fn from(e: Error) -> Self {
        let message = e.to_string();
        Self {
            inner: Arc::new(e),
            message,
        }
    }
}

#[cfg(feature = "sqlx-error")]
impl PartialEq for ErrFromSqlx {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message
    }

    fn ne(&self, other: &Self) -> bool {
        self.message != other.message
    }
}

#[cfg(feature = "sqlx-error")]
impl From<ErrFromSqlx> for String {
    fn from(e: ErrFromSqlx) -> Self {
        e.message
    }
}

#[cfg(feature = "sqlx-error")]
impl Display for ErrFromSqlx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DbErr {}

impl std::fmt::Display for DbErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Conn(s) => write!(f, "Connection Error: {}", s),
            Self::Exec(s) => write!(f, "Execution Error: {}", s),
            Self::Query(s) => write!(f, "Query Error: {}", s),
            Self::RecordNotFound(s) => write!(f, "RecordNotFound Error: {}", s),
            Self::Custom(s) => write!(f, "Custom Error: {}", s),
            Self::Type(s) => write!(f, "Type Error: {}", s),
            Self::Json(s) => write!(f, "Json Error: {}", s),
            Self::Migration(s) => write!(f, "Migration Error: {}", s),
            #[cfg(feature = "sqlx-error")]
            Self::Sqlx(s) => write!(f, "Sqlx Error: {}", s),
        }
    }
}

/// An error from a failed column operation when trying to convert the column to a string
#[derive(Debug, Clone)]
pub struct ColumnFromStrErr(pub String);

impl std::error::Error for ColumnFromStrErr {}

impl std::fmt::Display for ColumnFromStrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
