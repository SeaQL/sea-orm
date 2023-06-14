#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-dep"))]
pub use sqlx::error::Error as SqlxError;

#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-mysql"))]
pub use sqlx::mysql::MySqlDatabaseError as SqlxMySqlError;

#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-postgres"))]
pub use sqlx::postgres::PgDatabaseError as SqlxPostgresError;

#[cfg(all(feature = "sea-orm-internal", feature = "sqlx-sqlite"))]
pub use sqlx::sqlite::SqliteError as SqlxSqliteError;

use thiserror::Error;

/// An error from unsuccessful database operations
#[derive(Error, Debug)]
pub enum DbErr {
    /// This error can happen when the connection pool is fully-utilized
    #[error("Failed to acquire connection from pool")]
    ConnectionAcquire,
    /// Runtime type conversion error
    #[error("Error converting `{from}` into `{into}`: {source}")]
    TryIntoErr {
        /// From type
        from: &'static str,
        /// Into type
        into: &'static str,
        /// TryError
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// There was a problem with the database connection
    #[error("Connection Error: {0}")]
    Conn(#[source] RuntimeErr),
    /// An operation did not execute successfully
    #[error("Execution Error: {0}")]
    Exec(#[source] RuntimeErr),
    /// An error occurred while performing a query
    #[error("Query Error: {0}")]
    Query(#[source] RuntimeErr),
    /// Type error: the specified type cannot be converted from u64. This is not a runtime error.
    #[error("Type '{0}' cannot be converted from u64")]
    ConvertFromU64(&'static str),
    /// After an insert statement it was impossible to retrieve the last_insert_id
    #[error("Failed to unpack last_insert_id")]
    UnpackInsertId,
    /// When updating, a model should know its primary key to check
    /// if the record has been correctly updated, otherwise this error will occur
    #[error("Failed to get primary key from model")]
    UpdateGetPrimaryKey,
    /// The record was not found in the database
    #[error("RecordNotFound Error: {0}")]
    RecordNotFound(String),
    /// Thrown by `TryFrom<ActiveModel>`, which assumes all attributes are set/unchanged
    #[error("Attribute {0} is NotSet")]
    AttrNotSet(String),
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
    /// None of the records are inserted,
    /// that probably means all of them conflict with existing records in the table
    #[error("None of the records are inserted")]
    RecordNotInserted,
    /// None of the records are updated, that means a WHERE condition has no matches.
    /// May be the table is empty or the record does not exist
    #[error("None of the records are updated")]
    RecordNotUpdated,
}

/// Runtime error
#[derive(Error, Debug)]
pub enum RuntimeErr {
    /// SQLx Error
    #[cfg(feature = "sqlx-dep")]
    #[error("{0}")]
    SqlxError(sqlx::error::Error),
    /// Error generated from within SeaORM
    #[error("{0}")]
    Internal(String),
}

impl PartialEq for DbErr {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for DbErr {}

/// Error during `impl FromStr for Entity::Column`
#[derive(Error, Debug)]
#[error("Failed to match \"{0}\" as Column")]
pub struct ColumnFromStrErr(pub String);

#[allow(dead_code)]
pub(crate) fn conn_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Conn(RuntimeErr::Internal(s.to_string()))
}

#[allow(dead_code)]
pub(crate) fn exec_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Exec(RuntimeErr::Internal(s.to_string()))
}

#[allow(dead_code)]
pub(crate) fn query_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Query(RuntimeErr::Internal(s.to_string()))
}

#[allow(dead_code)]
pub(crate) fn type_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Type(s.to_string())
}

#[allow(dead_code)]
pub(crate) fn json_err<T>(s: T) -> DbErr
where
    T: ToString,
{
    DbErr::Json(s.to_string())
}

/// An error from unsuccessful SQL query
#[derive(Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SqlErr {
    /// error for inserting a record with a key that already exists in the table
    #[error("Cannot have record with same key")]
    UniqueConstraintViolation(),
    /// error for Foreign key is not primary key
    #[error("Cannot add non-primary key from other table")]
    ForeignKeyConstraintViolation(),
}

#[allow(dead_code)]
impl DbErr {
    /// converting generic DbErr from mysql to SqlErr
    #[cfg(any(
        feature = "sqlx-mysql",
        feature = "sqlx-postgres",
        feature = "sqlx-sqlite"
    ))]
    fn sql_err(&self) -> Option<SqlErr> {
        if let DbErr::Exec(RuntimeErr::SqlxError(sqlx::Error::Database(e)))
        | DbErr::Query(RuntimeErr::SqlxError(sqlx::Error::Database(e))) = self
        {
            #[cfg(feature = "sqlx-mysql")]
            {
                if e.try_downcast_ref::<SqlxMySqlError>().is_some() {
                    if e.code().unwrap().eq("1062") | e.code().unwrap().eq("1586") {
                        return Some(SqlErr::UniqueConstraintViolation());
                    };
                    if e.code().unwrap().eq("1452") {
                        return Some(SqlErr::ForeignKeyConstraintViolation());
                    };
                }
            }
            #[cfg(feature = "sqlx-postgres")]
            {
                if e.try_downcast_ref::<SqlxPostgresError>().is_some() {
                    if e.code().unwrap().eq("23505") {
                        return Some(SqlErr::UniqueConstraintViolation());
                    };
                    if e.code().unwrap().eq("23503") {
                        return Some(SqlErr::ForeignKeyConstraintViolation());
                    };
                }
            }
            #[cfg(feature = "sqlx-sqlite")]
            {
                if e.try_downcast_ref::<SqlxSqliteError>().is_some() {
                    if e.code().unwrap().eq("2067") {
                        return Some(SqlErr::UniqueConstraintViolation());
                    };
                    if e.code().unwrap().eq("787") {
                        return Some(SqlErr::ForeignKeyConstraintViolation());
                    };
                }
            }
        }
        None
    }
}
