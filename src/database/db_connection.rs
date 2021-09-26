use std::{future::Future, pin::Pin};
use crate::{DatabaseTransaction, DbBackend, DbErr, ExecResult, MockDatabaseConnection, QueryResult, QueryStream, Statement, TransactionError};
#[cfg(feature = "sqlx-dep")]
use sqlx::pool::PoolConnection;

pub(crate) enum InnerConnection<'a> {
    #[cfg(feature = "sqlx-mysql")]
    MySql(PoolConnection<sqlx::MySql>),
    #[cfg(feature = "sqlx-postgres")]
    Postgres(PoolConnection<sqlx::Postgres>),
    #[cfg(feature = "sqlx-sqlite")]
    Sqlite(PoolConnection<sqlx::Sqlite>),
    #[cfg(feature = "mock")]
    Mock(&'a MockDatabaseConnection),
    Transaction(Box<&'a DatabaseTransaction<'a>>),
}

#[async_trait::async_trait(?Send)]
pub trait ConnectionTrait<'a> {
    fn get_database_backend(&self) -> DbBackend;

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr>;

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr>;

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    async fn stream(&'a self, stmt: Statement) -> Result<QueryStream<'a>, DbErr>;

    async fn begin(&'a self) -> Result<DatabaseTransaction<'a>, DbErr>;

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E/*, Fut*/>(&'a self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction<'a>) -> Pin<Box<dyn Future<Output = Result<T, E>> + 'c>>,
        // F: FnOnce(&DatabaseTransaction<'a>) -> Fut + Send,
        // Fut: Future<Output = Result<T, E>> + Send,
        // T: Send,
        E: std::error::Error;

    fn is_mock_connection(&self) -> bool {
        false
    }
}
