use std::{pin::Pin, future::Future};
use crate::{DatabaseTransaction, ConnectionTrait, ExecResult, QueryResult, Statement, StatementBuilder, TransactionError, error::*};
use sea_query::{MysqlQueryBuilder, PostgresQueryBuilder, QueryBuilder, SqliteQueryBuilder};

pub enum DatabaseConnection {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlPoolConnection(crate::SqlxMySqlPoolConnection),
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgresPoolConnection(crate::SqlxPostgresPoolConnection),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlitePoolConnection(crate::SqlxSqlitePoolConnection),
    #[cfg(feature = "mock")]
    MockDatabaseConnection(crate::MockDatabaseConnection),
    Disconnected,
}

pub type DbConn = DatabaseConnection;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DatabaseBackend {
    MySql,
    Postgres,
    Sqlite,
}

pub type DbBackend = DatabaseBackend;

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

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.execute(stmt).await,
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_one(stmt).await,
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_all(stmt).await,
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E>(&self, _callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction<'_>) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>> + Send + Sync,
        T: Send,
        E: std::error::Error + Send,
    {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.transaction(_callback).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.transaction(_callback).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.transaction(_callback).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(_) => unimplemented!(), //TODO: support transaction in mock connection
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    #[cfg(feature = "mock")]
    fn is_mock_connection(&self) -> bool {
        match self {
            DatabaseConnection::MockDatabaseConnection(_) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "mock")]
impl DatabaseConnection {
    fn as_mock_connection(&self) -> &crate::MockDatabaseConnection {
        match self {
            DatabaseConnection::MockDatabaseConnection(mock_conn) => mock_conn,
            _ => panic!("not mock connection"),
        }
    }

    fn into_transaction_log(&self) -> Vec<crate::Transaction> {
        let mut mocker = self.as_mock_connection().get_mocker_mutex().lock().unwrap();
        mocker.drain_transaction_log()
    }
}

impl DbBackend {
    pub fn build<S>(&self, statement: &S) -> Statement
    where
        S: StatementBuilder,
    {
        statement.build(self)
    }

    pub fn get_query_builder(&self) -> Box<dyn QueryBuilder> {
        match self {
            Self::MySql => Box::new(MysqlQueryBuilder),
            Self::Postgres => Box::new(PostgresQueryBuilder),
            Self::Sqlite => Box::new(SqliteQueryBuilder),
        }
    }
}
