use crate::{
    error::*, AccessMode, ConnectionTrait, DatabaseTransaction, ExecResult, IsolationLevel,
    QueryResult, Statement, StatementBuilder, StreamTrait, TransactionError, TransactionTrait,
};
use sea_query::{MysqlQueryBuilder, PostgresQueryBuilder, QueryBuilder, SqliteQueryBuilder};
use std::{future::Future, pin::Pin};
use tracing::instrument;
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
    SqlxMySqlPoolConnection(crate::SqlxMySqlPoolConnection),
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
#[derive(Debug)]
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
                Self::SqlxMySqlPoolConnection(_) => "SqlxMySqlPoolConnection",
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
impl ConnectionTrait for DatabaseConnection {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(_) => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(_) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(_) => DbBackend::Sqlite,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.get_database_backend(),
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.execute(stmt),
            DatabaseConnection::Disconnected => {
                Err(DbErr::Conn(RuntimeErr::Internal("Disconnected".to_owned())))
            }
        }
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_one(stmt),
            DatabaseConnection::Disconnected => {
                Err(DbErr::Conn(RuntimeErr::Internal("Disconnected".to_owned())))
            }
        }
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_all(stmt),
            DatabaseConnection::Disconnected => {
                Err(DbErr::Conn(RuntimeErr::Internal("Disconnected".to_owned())))
            }
        }
    }

    #[cfg(feature = "mock")]
    fn is_mock_connection(&self) -> bool {
        matches!(self, DatabaseConnection::MockDatabaseConnection(_))
    }
}

#[async_trait::async_trait]
impl StreamTrait for DatabaseConnection {
    type Stream<'a> = crate::QueryStream;

    #[instrument(level = "trace")]
    #[allow(unused_variables, unreachable_code)]
    fn stream<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        Box::pin(async move {
            Ok(match self {
                #[cfg(feature = "sqlx-mysql")]
                DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.stream(stmt).await?,
                #[cfg(feature = "sqlx-postgres")]
                DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.stream(stmt).await?,
                #[cfg(feature = "sqlx-sqlite")]
                DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.stream(stmt).await?,
                #[cfg(feature = "mock")]
                DatabaseConnection::MockDatabaseConnection(conn) => {
                    crate::QueryStream::from((Arc::clone(conn), stmt, None))
                }
                DatabaseConnection::Disconnected => panic!("Disconnected"),
            })
        })
    }
}

#[async_trait::async_trait]
impl TransactionTrait for DatabaseConnection {
    #[instrument(level = "trace")]
    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.begin(None, None).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.begin(None, None).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.begin(None, None).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => {
                DatabaseTransaction::new_mock(Arc::clone(conn), None).await
            }
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    #[instrument(level = "trace")]
    async fn begin_with_config(
        &self,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => {
                conn.begin(_isolation_level, _access_mode).await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
                conn.begin(_isolation_level, _access_mode).await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => {
                conn.begin(_isolation_level, _access_mode).await
            }
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => {
                DatabaseTransaction::new_mock(Arc::clone(conn), None).await
            }
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(_callback))]
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
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => {
                conn.transaction(_callback, None, None).await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
                conn.transaction(_callback, None, None).await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => {
                conn.transaction(_callback, None, None).await
            }
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_mock(Arc::clone(conn), None)
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(_callback))]
    async fn transaction_with_config<F, T, E>(
        &self,
        _callback: F,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
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
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
                    .await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
                    .await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
                    .await
            }
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_mock(Arc::clone(conn), None)
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
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
    /// Sets a callback to metric this connection
    pub fn set_metric_callback<F>(&mut self, _callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + Send + Sync + 'static,
    {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            _ => {}
        }
    }

    /// Explicitly close the database connection
    pub async fn close(self) -> Result<(), DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.close().await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.close().await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.close().await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(_) => {
                // Nothing to cleanup, we just consume the `DatabaseConnection`
                Ok(())
            }
            DatabaseConnection::Disconnected => {
                Err(DbErr::Conn(RuntimeErr::Internal("Disconnected".to_owned())))
            }
        }
    }
}

#[cfg(feature = "sea-orm-internal")]
impl DatabaseConnection {
    /// Get [sqlx::MySqlPool]
    #[cfg(feature = "sqlx-mysql")]
    pub fn get_mysql_connection_pool(&self) -> &sqlx::MySqlPool {
        match self {
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => &conn.pool,
            _ => panic!("Not MySQL Connection"),
        }
    }

    /// Get [sqlx::PgPool]
    #[cfg(feature = "sqlx-postgres")]
    pub fn get_postgres_connection_pool(&self) -> &sqlx::PgPool {
        match self {
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => &conn.pool,
            _ => panic!("Not Postgres Connection"),
        }
    }

    /// Get [sqlx::SqlitePool]
    #[cfg(feature = "sqlx-sqlite")]
    pub fn get_sqlite_connection_pool(&self) -> &sqlx::SqlitePool {
        match self {
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => &conn.pool,
            _ => panic!("Not SQLite Connection"),
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

    /// Check if the database supports `RETURNING` syntax on insert and update
    pub fn support_returning(&self) -> bool {
        matches!(self, Self::Postgres)
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
