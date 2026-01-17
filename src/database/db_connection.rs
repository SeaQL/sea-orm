use crate::{
    AccessMode, ConnectionTrait, DatabaseTransaction, ExecResult, IsolationLevel, QueryResult,
    Schema, SchemaBuilder, Statement, StatementBuilder, StreamTrait, TransactionError,
    TransactionTrait, error::*,
};
use std::{fmt::Debug, future::Future, pin::Pin};
use tracing::instrument;
use url::Url;

#[cfg(feature = "sqlx-dep")]
use sqlx::pool::PoolConnection;

#[cfg(feature = "rusqlite")]
use crate::driver::rusqlite::{RusqliteInnerConnection, RusqliteSharedConnection};

#[cfg(any(feature = "mock", feature = "proxy"))]
use std::sync::Arc;

/// Handle a database connection depending on the backend enabled by the feature
/// flags. This creates a connection pool internally (for SQLx connections),
/// and so is cheap to clone.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct DatabaseConnection {
    /// `DatabaseConnection` used to be a enum. Now it's moved into inner,
    /// because we have to attach other contexts.
    pub inner: DatabaseConnectionType,
    #[cfg(feature = "rbac")]
    pub(crate) rbac: crate::RbacEngineMount,
}

/// The underlying database connection type.
#[derive(Clone)]
pub enum DatabaseConnectionType {
    /// MySql database connection pool
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlPoolConnection(crate::SqlxMySqlPoolConnection),

    /// PostgreSQL database connection pool
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgresPoolConnection(crate::SqlxPostgresPoolConnection),

    /// SQLite database connection pool
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlitePoolConnection(crate::SqlxSqlitePoolConnection),

    /// SQLite database connection sharable across threads
    #[cfg(feature = "rusqlite")]
    RusqliteSharedConnection(RusqliteSharedConnection),

    /// Mock database connection useful for testing
    #[cfg(feature = "mock")]
    MockDatabaseConnection(Arc<crate::MockDatabaseConnection>),

    /// Proxy database connection
    #[cfg(feature = "proxy")]
    ProxyDatabaseConnection(Arc<crate::ProxyDatabaseConnection>),

    /// The connection has never been established
    Disconnected,
}

/// The same as a [DatabaseConnection]
pub type DbConn = DatabaseConnection;

impl Default for DatabaseConnection {
    fn default() -> Self {
        DatabaseConnectionType::Disconnected.into()
    }
}

impl From<DatabaseConnectionType> for DatabaseConnection {
    fn from(inner: DatabaseConnectionType) -> Self {
        Self {
            inner,
            #[cfg(feature = "rbac")]
            rbac: Default::default(),
        }
    }
}

/// The type of database backend for real world databases.
/// This is enabled by feature flags as specified in the crate documentation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DatabaseBackend {
    /// A MySQL backend
    MySql,
    /// A PostgreSQL backend
    Postgres,
    /// A SQLite backend
    Sqlite,
}

/// A shorthand for [DatabaseBackend].
pub type DbBackend = DatabaseBackend;

#[derive(Debug)]
pub(crate) enum InnerConnection {
    #[cfg(feature = "sqlx-mysql")]
    MySql(PoolConnection<sqlx::MySql>),
    #[cfg(feature = "sqlx-postgres")]
    Postgres(PoolConnection<sqlx::Postgres>),
    #[cfg(feature = "sqlx-sqlite")]
    Sqlite(PoolConnection<sqlx::Sqlite>),
    #[cfg(feature = "rusqlite")]
    Rusqlite(RusqliteInnerConnection),
    #[cfg(feature = "mock")]
    Mock(Arc<crate::MockDatabaseConnection>),
    #[cfg(feature = "proxy")]
    Proxy(Arc<crate::ProxyDatabaseConnection>),
}

impl Debug for DatabaseConnectionType {
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
                #[cfg(feature = "rusqlite")]
                Self::RusqliteSharedConnection(_) => "RusqliteSharedConnection",
                #[cfg(feature = "mock")]
                Self::MockDatabaseConnection(_) => "MockDatabaseConnection",
                #[cfg(feature = "proxy")]
                Self::ProxyDatabaseConnection(_) => "ProxyDatabaseConnection",
                Self::Disconnected => "Disconnected",
            }
        )
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for DatabaseConnection {
    fn get_database_backend(&self) -> DbBackend {
        self.get_database_backend()
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        super::tracing_spans::with_db_span!(
            "sea_orm.execute",
            self.get_database_backend(),
            stmt.sql.as_str(),
            record_stmt = true,
            async {
                match &self.inner {
                    #[cfg(feature = "sqlx-mysql")]
                    DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                        conn.execute(stmt).await
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                        conn.execute(stmt).await
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                        conn.execute(stmt).await
                    }
                    #[cfg(feature = "rusqlite")]
                    DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.execute(stmt),
                    #[cfg(feature = "mock")]
                    DatabaseConnectionType::MockDatabaseConnection(conn) => conn.execute(stmt),
                    #[cfg(feature = "proxy")]
                    DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                        conn.execute(stmt).await
                    }
                    DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
                }
            }
        )
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        super::tracing_spans::with_db_span!(
            "sea_orm.execute_unprepared",
            self.get_database_backend(),
            sql,
            record_stmt = false,
            async {
                match &self.inner {
                    #[cfg(feature = "sqlx-mysql")]
                    DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                        conn.execute_unprepared(sql).await
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                        conn.execute_unprepared(sql).await
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                        conn.execute_unprepared(sql).await
                    }
                    #[cfg(feature = "rusqlite")]
                    DatabaseConnectionType::RusqliteSharedConnection(conn) => {
                        conn.execute_unprepared(sql)
                    }
                    #[cfg(feature = "mock")]
                    DatabaseConnectionType::MockDatabaseConnection(conn) => {
                        let db_backend = conn.get_database_backend();
                        let stmt = Statement::from_string(db_backend, sql);
                        conn.execute(stmt)
                    }
                    #[cfg(feature = "proxy")]
                    DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                        let db_backend = conn.get_database_backend();
                        let stmt = Statement::from_string(db_backend, sql);
                        conn.execute(stmt).await
                    }
                    DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
                }
            }
        )
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        super::tracing_spans::with_db_span!(
            "sea_orm.query_one",
            self.get_database_backend(),
            stmt.sql.as_str(),
            record_stmt = true,
            async {
                match &self.inner {
                    #[cfg(feature = "sqlx-mysql")]
                    DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                        conn.query_one(stmt).await
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                        conn.query_one(stmt).await
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                        conn.query_one(stmt).await
                    }
                    #[cfg(feature = "rusqlite")]
                    DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.query_one(stmt),
                    #[cfg(feature = "mock")]
                    DatabaseConnectionType::MockDatabaseConnection(conn) => conn.query_one(stmt),
                    #[cfg(feature = "proxy")]
                    DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                        conn.query_one(stmt).await
                    }
                    DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
                }
            }
        )
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        super::tracing_spans::with_db_span!(
            "sea_orm.query_all",
            self.get_database_backend(),
            stmt.sql.as_str(),
            record_stmt = true,
            async {
                match &self.inner {
                    #[cfg(feature = "sqlx-mysql")]
                    DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                        conn.query_all(stmt).await
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                        conn.query_all(stmt).await
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                        conn.query_all(stmt).await
                    }
                    #[cfg(feature = "rusqlite")]
                    DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.query_all(stmt),
                    #[cfg(feature = "mock")]
                    DatabaseConnectionType::MockDatabaseConnection(conn) => conn.query_all(stmt),
                    #[cfg(feature = "proxy")]
                    DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                        conn.query_all(stmt).await
                    }
                    DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
                }
            }
        )
    }

    #[cfg(feature = "mock")]
    fn is_mock_connection(&self) -> bool {
        matches!(
            self,
            DatabaseConnection {
                inner: DatabaseConnectionType::MockDatabaseConnection(_),
                ..
            }
        )
    }
}

#[async_trait::async_trait]
impl StreamTrait for DatabaseConnection {
    type Stream<'a> = crate::QueryStream;

    fn get_database_backend(&self) -> DbBackend {
        self.get_database_backend()
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    fn stream_raw<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>> {
        Box::pin(async move {
            match &self.inner {
                #[cfg(feature = "sqlx-mysql")]
                DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => conn.stream(stmt).await,
                #[cfg(feature = "sqlx-postgres")]
                DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => conn.stream(stmt).await,
                #[cfg(feature = "sqlx-sqlite")]
                DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => conn.stream(stmt).await,
                #[cfg(feature = "rusqlite")]
                DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.stream(stmt),
                #[cfg(feature = "mock")]
                DatabaseConnectionType::MockDatabaseConnection(conn) => {
                    Ok(crate::QueryStream::from((Arc::clone(conn), stmt, None)))
                }
                #[cfg(feature = "proxy")]
                DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                    Ok(crate::QueryStream::from((Arc::clone(conn), stmt, None)))
                }
                DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
            }
        })
    }
}

#[async_trait::async_trait]
impl TransactionTrait for DatabaseConnection {
    type Transaction = DatabaseTransaction;

    #[instrument(level = "trace")]
    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => conn.begin(None, None).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                conn.begin(None, None).await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => conn.begin(None, None).await,
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.begin(None, None),
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(conn) => {
                DatabaseTransaction::new_mock(Arc::clone(conn), None).await
            }
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                DatabaseTransaction::new_proxy(conn.clone(), None).await
            }
            DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
        }
    }

    #[instrument(level = "trace")]
    async fn begin_with_config(
        &self,
        _isolation_level: Option<IsolationLevel>,
        _access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                conn.begin(_isolation_level, _access_mode).await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                conn.begin(_isolation_level, _access_mode).await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                conn.begin(_isolation_level, _access_mode).await
            }
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => {
                conn.begin(_isolation_level, _access_mode)
            }
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(conn) => {
                DatabaseTransaction::new_mock(Arc::clone(conn), None).await
            }
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                DatabaseTransaction::new_proxy(conn.clone(), None).await
            }
            DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
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
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                conn.transaction(_callback, None, None).await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                conn.transaction(_callback, None, None).await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                conn.transaction(_callback, None, None).await
            }
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => {
                conn.transaction(_callback, None, None)
            }
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_mock(Arc::clone(conn), None)
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_proxy(conn.clone(), None)
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected").into()),
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
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
                    .await
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
                    .await
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
                    .await
            }
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => {
                conn.transaction(_callback, _isolation_level, _access_mode)
            }
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_mock(Arc::clone(conn), None)
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(conn) => {
                let transaction = DatabaseTransaction::new_proxy(conn.clone(), None)
                    .await
                    .map_err(TransactionError::Connection)?;
                transaction.run(_callback).await
            }
            DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected").into()),
        }
    }
}

#[cfg(feature = "mock")]
impl DatabaseConnection {
    /// Generate a database connection for testing the Mock database
    ///
    /// # Panics
    ///
    /// Panics if [DbConn] is not a mock connection.
    pub fn as_mock_connection(&self) -> &crate::MockDatabaseConnection {
        match &self.inner {
            DatabaseConnectionType::MockDatabaseConnection(mock_conn) => mock_conn,
            _ => panic!("Not mock connection"),
        }
    }

    /// Get the transaction log as a collection Vec<[crate::Transaction]>
    ///
    /// # Panics
    ///
    /// Panics if the mocker mutex is being held by another thread.
    pub fn into_transaction_log(self) -> Vec<crate::Transaction> {
        let mut mocker = self
            .as_mock_connection()
            .get_mocker_mutex()
            .lock()
            .expect("Fail to acquire mocker");
        mocker.drain_transaction_log()
    }
}

#[cfg(feature = "proxy")]
impl DatabaseConnection {
    /// Generate a database connection for testing the Proxy database
    ///
    /// # Panics
    ///
    /// Panics if [DbConn] is not a proxy connection.
    pub fn as_proxy_connection(&self) -> &crate::ProxyDatabaseConnection {
        match &self.inner {
            DatabaseConnectionType::ProxyDatabaseConnection(proxy_conn) => proxy_conn,
            _ => panic!("Not proxy connection"),
        }
    }
}

#[cfg(feature = "rbac")]
impl DatabaseConnection {
    /// Load RBAC data from the same database as this connection and setup RBAC engine.
    /// If the RBAC engine already exists, it will be replaced.
    pub async fn load_rbac(&self) -> Result<(), DbErr> {
        self.load_rbac_from(self).await
    }

    /// Load RBAC data from the given database connection and setup RBAC engine.
    /// This could be from another database.
    pub async fn load_rbac_from(&self, db: &DbConn) -> Result<(), DbErr> {
        let engine = crate::rbac::RbacEngine::load_from(db).await?;
        self.rbac.replace(engine);
        Ok(())
    }

    /// Replace the internal RBAC engine.
    pub fn replace_rbac(&self, engine: crate::rbac::RbacEngine) {
        self.rbac.replace(engine);
    }

    /// Create a restricted connection with access control specific for the user.
    pub fn restricted_for(
        &self,
        user_id: crate::rbac::RbacUserId,
    ) -> Result<crate::RestrictedConnection, DbErr> {
        if self.rbac.is_some() {
            Ok(crate::RestrictedConnection {
                user_id,
                conn: self.clone(),
            })
        } else {
            Err(DbErr::RbacError("engine not set up".into()))
        }
    }
}

impl DatabaseConnection {
    /// Get the database backend for this connection
    ///
    /// # Panics
    ///
    /// Panics if [DatabaseConnection] is `Disconnected`.
    pub fn get_database_backend(&self) -> DbBackend {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(_) => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(_) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(_) => DbBackend::Sqlite,
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(_) => DbBackend::Sqlite,
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(conn) => conn.get_database_backend(),
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(conn) => conn.get_database_backend(),
            DatabaseConnectionType::Disconnected => panic!("Disconnected"),
        }
    }

    /// Creates a [`SchemaBuilder`] for this backend
    pub fn get_schema_builder(&self) -> SchemaBuilder {
        Schema::new(self.get_database_backend()).builder()
    }

    #[cfg(feature = "entity-registry")]
    #[cfg_attr(docsrs, doc(cfg(feature = "entity-registry")))]
    /// Builds a schema for all the entites in the given module
    pub fn get_schema_registry(&self, prefix: &str) -> SchemaBuilder {
        let schema = Schema::new(self.get_database_backend());
        crate::EntityRegistry::build_schema(schema, prefix)
    }

    /// Sets a callback to metric this connection
    pub fn set_metric_callback<F>(&mut self, _callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + Send + Sync + 'static,
    {
        match &mut self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => {
                conn.set_metric_callback(_callback)
            }
            _ => {}
        }
    }

    /// Checks if a connection to the database is still valid.
    pub async fn ping(&self) -> Result<(), DbErr> {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => conn.ping().await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => conn.ping().await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => conn.ping().await,
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.ping(),
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(conn) => conn.ping(),
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(conn) => conn.ping().await,
            DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
        }
    }

    /// Explicitly close the database connection.
    /// See [`Self::close_by_ref`] for usage with references.
    pub async fn close(self) -> Result<(), DbErr> {
        self.close_by_ref().await
    }

    /// Explicitly close the database connection
    pub async fn close_by_ref(&self) -> Result<(), DbErr> {
        match &self.inner {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => conn.close_by_ref().await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => conn.close_by_ref().await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => conn.close_by_ref().await,
            #[cfg(feature = "rusqlite")]
            DatabaseConnectionType::RusqliteSharedConnection(conn) => conn.close_by_ref(),
            #[cfg(feature = "mock")]
            DatabaseConnectionType::MockDatabaseConnection(_) => {
                // Nothing to cleanup, we just consume the `DatabaseConnection`
                Ok(())
            }
            #[cfg(feature = "proxy")]
            DatabaseConnectionType::ProxyDatabaseConnection(_) => {
                // Nothing to cleanup, we just consume the `DatabaseConnection`
                Ok(())
            }
            DatabaseConnectionType::Disconnected => Err(conn_err("Disconnected")),
        }
    }
}

impl DatabaseConnection {
    /// Get [sqlx::MySqlPool]
    ///
    /// # Panics
    ///
    /// Panics if [DbConn] is not a MySQL connection.
    #[cfg(feature = "sqlx-mysql")]
    pub fn get_mysql_connection_pool(&self) -> &sqlx::MySqlPool {
        match &self.inner {
            DatabaseConnectionType::SqlxMySqlPoolConnection(conn) => &conn.pool,
            _ => panic!("Not MySQL Connection"),
        }
    }

    /// Get [sqlx::PgPool]
    ///
    /// # Panics
    ///
    /// Panics if [DbConn] is not a Postgres connection.
    #[cfg(feature = "sqlx-postgres")]
    pub fn get_postgres_connection_pool(&self) -> &sqlx::PgPool {
        match &self.inner {
            DatabaseConnectionType::SqlxPostgresPoolConnection(conn) => &conn.pool,
            _ => panic!("Not Postgres Connection"),
        }
    }

    /// Get [sqlx::SqlitePool]
    ///
    /// # Panics
    ///
    /// Panics if [DbConn] is not a SQLite connection.
    #[cfg(feature = "sqlx-sqlite")]
    pub fn get_sqlite_connection_pool(&self) -> &sqlx::SqlitePool {
        match &self.inner {
            DatabaseConnectionType::SqlxSqlitePoolConnection(conn) => &conn.pool,
            _ => panic!("Not SQLite Connection"),
        }
    }
}

impl DbBackend {
    /// Check if the URI is the same as the specified database backend.
    /// Returns true if they match.
    ///
    /// # Panics
    ///
    /// Panics if `base_url` cannot be parsed as `Url`.
    pub fn is_prefix_of(self, base_url: &str) -> bool {
        let base_url_parsed = Url::parse(base_url).expect("Fail to parse database URL");
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

    /// Check if the database supports `RETURNING` syntax on insert and update
    pub fn support_returning(&self) -> bool {
        match self {
            Self::Postgres => true,
            Self::Sqlite if cfg!(feature = "sqlite-use-returning-for-3_35") => true,
            Self::MySql if cfg!(feature = "mariadb-use-returning") => true,
            _ => false,
        }
    }

    /// A getter for database dependent boolean value
    pub fn boolean_value(&self, boolean: bool) -> sea_query::Value {
        match self {
            Self::MySql | Self::Postgres | Self::Sqlite => boolean.into(),
        }
    }

    /// Get the display string for this enum
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseBackend::MySql => "MySql",
            DatabaseBackend::Postgres => "Postgres",
            DatabaseBackend::Sqlite => "Sqlite",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DatabaseConnection;

    #[cfg(not(feature = "sync"))]
    #[test]
    fn assert_database_connection_traits() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<DatabaseConnection>();
    }
}
