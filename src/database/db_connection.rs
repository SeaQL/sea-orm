use crate::{
    error::*, ConnectionTrait, DatabaseTransaction, ExecResult, QueryResult, Statement,
    StatementBuilder, TransactionError,
};
use sea_query::{MysqlQueryBuilder, PostgresQueryBuilder, QueryBuilder, SqliteQueryBuilder};
use std::{future::Future, pin::Pin};
use url::Url;

#[cfg(feature = "sqlx-dep")]
use sqlx::pool::PoolConnection;

#[cfg(feature = "mock")]
use std::sync::Arc;

/// Handle a database connection depending on the backend
/// enabled by the feature flags. This creates a database pool.
#[cfg_attr(not(feature = "mock"), derive(Clone))]
pub enum DatabaseConnection {
    /// Create a MYSQL database connection and pool
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlPoolConnection {
        /// The SQLx MySQL pool
        conn: crate::SqlxMySqlPoolConnection,
        /// The MySQL version
        version: String,
        /// The flag indicating whether `RETURNING` syntax is supported
        support_returning: bool,
    },
    /// Create a  PostgreSQL database connection and pool
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgresPoolConnection(crate::SqlxPostgresPoolConnection),
    /// Create a  SQLite database connection and pool
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlitePoolConnection(crate::SqlxSqlitePoolConnection),
    /// Create a  Mock database connection useful for testing
    #[cfg(feature = "mock")]
    MockDatabaseConnection(Arc<crate::MockDatabaseConnection>),
    /// The connection to the database has been severed
    Disconnected,
}

/// The same as a [DatabaseConnection]
pub type DbConn = DatabaseConnection;

/// The type of database backend for real world databases.
/// This is enabled by feature flags as specified in the crate documentation
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DatabaseBackend {
    /// A MySQL backend
    MySql,
    /// A PostgreSQL backend
    Postgres,
    /// A SQLite backend
    Sqlite,
}

/// The same as [DatabaseBackend] just shorter :)
pub type DbBackend = DatabaseBackend;
pub(crate) enum InnerConnection {
    #[cfg(feature = "sqlx-mysql")]
    MySql(PoolConnection<sqlx::MySql>),
    #[cfg(feature = "sqlx-postgres")]
    Postgres(PoolConnection<sqlx::Postgres>),
    #[cfg(feature = "sqlx-sqlite")]
    Sqlite(PoolConnection<sqlx::Sqlite>),
    #[cfg(feature = "mock")]
    Mock(std::sync::Arc<crate::MockDatabaseConnection>),
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl std::fmt::Debug for DatabaseConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                #[cfg(feature = "sqlx-mysql")]
                Self::SqlxMySqlPoolConnection { .. } => "SqlxMySqlPoolConnection",
                #[cfg(feature = "sqlx-postgres")]
                Self::SqlxPostgresPoolConnection(_) => "SqlxPostgresPoolConnection",
                #[cfg(feature = "sqlx-sqlite")]
                Self::SqlxSqlitePoolConnection(_) => "SqlxSqlitePoolConnection",
                #[cfg(feature = "mock")]
                Self::MockDatabaseConnection(_) => "MockDatabaseConnection",
                Self::Disconnected => "Disconnected",
            }
        )
    }
}

#[async_trait::async_trait]
impl<'a> ConnectionTrait<'a> for DatabaseConnection {
    type Stream = crate::QueryStream;

    fn get_database_backend(&self) -> DbBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { .. } => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(_) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(_) => DbBackend::Sqlite,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.get_database_backend(),
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { conn, .. } => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.execute(stmt),
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { conn, .. } => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_one(stmt),
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { conn, .. } => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_all(stmt),
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    fn stream(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, DbErr>> + 'a>> {
        Box::pin(async move {
            Ok(match self {
                #[cfg(feature = "sqlx-mysql")]
                DatabaseConnection::SqlxMySqlPoolConnection { conn, .. } => {
                    conn.stream(stmt).await?
                }
                #[cfg(feature = "sqlx-postgres")]
                DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.stream(stmt).await?,
                #[cfg(feature = "sqlx-sqlite")]
                DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.stream(stmt).await?,
                #[cfg(feature = "mock")]
                DatabaseConnection::MockDatabaseConnection(conn) => {
                    crate::QueryStream::from((Arc::clone(conn), stmt))
                }
                DatabaseConnection::Disconnected => panic!("Disconnected"),
            })
        })
    }

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { conn, .. } => conn.begin().await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.begin().await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.begin().await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => {
                DatabaseTransaction::new_mock(Arc::clone(conn)).await
            }
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E>(&self, _callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { conn, .. } => {
                conn.transaction(_callback).await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
                conn.transaction(_callback).await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.transaction(_callback).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_mock(Arc::clone(conn))
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    fn support_returning(&self) -> bool {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { support_returning, .. } => *support_returning,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(_) => true,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(_) => false,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => match conn.get_database_backend() {
                DbBackend::MySql => false,
                DbBackend::Postgres => true,
                DbBackend::Sqlite => false,
            },
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    #[cfg(feature = "mock")]
    fn is_mock_connection(&self) -> bool {
        matches!(self, DatabaseConnection::MockDatabaseConnection(_))
    }
}

#[cfg(feature = "mock")]
impl DatabaseConnection {
    /// Generate a database connection for testing the Mock database
    pub fn as_mock_connection(&self) -> &crate::MockDatabaseConnection {
        match self {
            DatabaseConnection::MockDatabaseConnection(mock_conn) => mock_conn,
            _ => panic!("not mock connection"),
        }
    }

    /// Get the transaction log as a collection  Vec<[crate::Transaction]>
    pub fn into_transaction_log(self) -> Vec<crate::Transaction> {
        let mut mocker = self.as_mock_connection().get_mocker_mutex().lock().unwrap();
        mocker.drain_transaction_log()
    }
}

impl DatabaseConnection {
    /// Get database version
    pub fn db_version(&self) -> String {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { version, .. } => version.to_string(),
            // #[cfg(feature = "sqlx-postgres")]
            // DatabaseConnection::SqlxPostgresPoolConnection(conn) => ,
            // #[cfg(feature = "sqlx-sqlite")]
            // DatabaseConnection::SqlxSqlitePoolConnection(conn) => ,
            // #[cfg(feature = "mock")]
            // DatabaseConnection::MockDatabaseConnection(conn) => ,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
            _ => unimplemented!(),
        }
    }

    /// Check if database supports `RETURNING`
    pub fn db_support_returning(&self) -> bool {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection { support_returning, .. } => *support_returning,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(_) => true,
            // #[cfg(feature = "sqlx-sqlite")]
            // DatabaseConnection::SqlxSqlitePoolConnection(conn) => ,
            // #[cfg(feature = "mock")]
            // DatabaseConnection::MockDatabaseConnection(conn) => ,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
            _ => unimplemented!(),
        }
    }
}

impl DbBackend {
    /// Check if the URI is the same as the specified database backend.
    /// Returns true if they match.
    pub fn is_prefix_of(self, base_url: &str) -> bool {
        let base_url_parsed = Url::parse(base_url).unwrap();
        match self {
            Self::Postgres => {
                base_url_parsed.scheme() == "postgres" || base_url_parsed.scheme() == "postgresql"
            }
            Self::MySql => base_url_parsed.scheme() == "mysql",
            Self::Sqlite => base_url_parsed.scheme() == "sqlite",
        }
    }

    /// Build an SQL [Statement]
    pub fn build<S>(&self, statement: &S) -> Statement
    where
        S: StatementBuilder,
    {
        statement.build(self)
    }

    /// A helper for building SQL queries
    pub fn get_query_builder(&self) -> Box<dyn QueryBuilder> {
        match self {
            Self::MySql => Box::new(MysqlQueryBuilder),
            Self::Postgres => Box::new(PostgresQueryBuilder),
            Self::Sqlite => Box::new(SqliteQueryBuilder),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DatabaseConnection;

    #[test]
    fn assert_database_connection_traits() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<DatabaseConnection>();
    }
}
