#[cfg(feature = "sqlx-dep")]
use sqlx::error::Error as SqlxError;
use std::sync::Arc;

#[cfg(not(feature = "sqlx-dep"))]
type SqlxError = ();

/// An error from unsuccessful database operations
#[derive(Debug, Clone)]
pub enum DbErr {
    /// There was a problem with the database connection
    Conn(String),
    /// An operation did not execute successfully
    Exec(String, Option<Arc<SqlxError>>),
    /// An error occurred while performing a query
    Query(String, Option<Arc<SqlxError>>),
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
}

impl PartialEq for DbErr {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl std::error::Error for DbErr {}

#[cfg(feature = "sqlx-dep")]
impl DbErr {
    /// provides the underlying error from sqlx, if available
    pub fn sqlx_error(&self) -> Option<&SqlxError> {
        match self {
            DbErr::Exec(_, source) => source.as_ref(),
            DbErr::Query(_, source) => source.as_ref(),
            _ => None,
        }
        .map(|s| s.as_ref())
    }
}

impl std::fmt::Display for DbErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Conn(s, ..) => write!(f, "Connection Error: {}", s),
            Self::Exec(s, ..) => write!(f, "Execution Error: {}", s),
            Self::Query(s, ..) => write!(f, "Query Error: {}", s),
            Self::RecordNotFound(s) => write!(f, "RecordNotFound Error: {}", s),
            Self::Custom(s) => write!(f, "Custom Error: {}", s),
            Self::Type(s) => write!(f, "Type Error: {}", s),
            Self::Json(s) => write!(f, "Json Error: {}", s),
            Self::Migration(s) => write!(f, "Migration Error: {}", s),
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
