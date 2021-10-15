use crate::{
    DatabaseTransaction, DbBackend, DbErr, ExecResult, QueryResult, Statement, TransactionError,
};
use futures::Stream;
use std::{future::Future, pin::Pin};

#[async_trait::async_trait]
pub trait ConnectionTrait<'a>: Sync {
    type Stream: Stream<Item = Result<QueryResult, DbErr>>;

    fn get_database_backend(&self) -> DbBackend;

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr>;

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr>;

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    fn stream(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, DbErr>> + 'a>>;

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr>;

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::error::Error + Send;

    fn is_mock_connection(&self) -> bool {
        false
    }
}
