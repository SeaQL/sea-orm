#[cfg(feature = "sqlx-dep")]
use sqlx::error::Error as SqlxError;
use thiserror::Error;

/// An error from unsuccessful database operations
#[derive(Error, Debug)]
pub enum DbErr {
    /// This error happens, when a pool was not able to create a connection
    #[error("Failed to acquire connection from pool.")]
    ConnFromPool,
    /// Error in case of invalid type conversion attempts
    #[error("fail to convert '{0}' into '{1}'")]
    CannotConvertInto(String, String),
    /// Error in case of invalid type conversion from an u64
    #[error("{0} cannot be converted from u64")]
    ConvertFromU64(String),
    /// After an insert statement it was impossible to retrieve the last_insert_id
    #[error("Fail to unpack last_insert_id")]
    InsertCouldNotUnpackInsertId,
    /// When updating, a model should know it's primary key to check
    /// if the record has been correctly updated, otherwise this error will occur
    #[error("Fail to get primary key from model")]
    UpdateCouldNotGetPrimaryKey,
    /// There was a problem with the database connection
    #[error("Connection Error: {0}")]
    Conn(String),
    /// There was a problem with the database connection from sqlx
    #[cfg(feature = "sqlx-dep")]
    #[error("Connection Error: {0}")]
    ConnSqlX(#[source] SqlxError),
    /// An operation did not execute successfully
    #[cfg(feature = "sqlx-dep")]
    #[error("Execution Error: {0}")]
    ExecSqlX(#[source] SqlxError),
    /// An error occurred while performing a query, with more details from sqlx
    #[cfg(feature = "sqlx-dep")]
    #[error("Query Error: {0}")]
    QuerySqlX(#[source] SqlxError),
    /// An error occurred while performing a query
    #[error("Query Error: {0}")]
    Query(String),
    /// The record was not found in the database
    #[error("RecordNotFound Error: {0}")]
    RecordNotFound(String),
    /// A custom error
    #[error("Custom Error: {0}")]
    Custom(String),
    /// Error occurred while parsing value as target type
    #[error("Type Error: {0}")]
    Type(String),
    /// Error occurred while parsing json value as target type
    #[error("Json Error: {0}")]
    Json(String),
    /// A migration error
    #[error("Migration Error: {0}")]
    Migration(String),
}

impl PartialEq for DbErr {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
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
