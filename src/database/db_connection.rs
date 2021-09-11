use std::future::Future;
use crate::{DatabaseTransaction, DbBackend, DbErr, ExecResult, QueryResult, Statement, TransactionError};

#[async_trait::async_trait]
pub trait DbConnection {
    fn get_database_backend(&self) -> DbBackend;

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr>;

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr>;

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E, Fut>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut + Send,
        Fut: Future<Output=Result<T, E>> + Send,
        E: std::error::Error;

    #[cfg(feature = "mock")]
    fn as_mock_connection(&self) -> &crate::MockDatabaseConnection;

    #[cfg(not(feature = "mock"))]
    fn as_mock_connection(&self) -> Option<bool> {
        None
    }

    #[cfg(feature = "mock")]
    fn into_transaction_log(&self) -> Vec<crate::Transaction> {
        let mut mocker = self.as_mock_connection().get_mocker_mutex().lock().unwrap();
        mocker.drain_transaction_log()
    }
}
