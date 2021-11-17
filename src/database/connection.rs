use crate::{
    DatabaseTransaction, DbBackend, DbErr, ExecResult, QueryResult, Statement, TransactionError,
};
use futures::Stream;
use std::{future::Future, pin::Pin};

/// Creates constraints for any structure that can create a database connection
/// and execute SQL statements
#[async_trait::async_trait]
pub trait ConnectionTrait<'a>: Sync {
    /// Create a stream for the [QueryResult]
    type Stream: Stream<Item = Result<QueryResult, DbErr>>;

    /// Fetch the database backend as specified in [DbBackend].
    /// This depends on feature flags enabled.
    fn get_database_backend(&self) -> DbBackend;

    /// Execute a [Statement]
    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr>;

    /// Execute a [Statement] and return a query
    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr>;

    /// Execute a [Statement] and return a collection Vec<[QueryResult]> on success
    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    /// Execute a [Statement] and return a stream of results
    fn stream(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, DbErr>> + 'a>>;

    /// Execute SQL `BEGIN` transaction.
    /// Returns a Transaction that can be committed or rolled back
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

    /// Check if the connection supports `RETURNING` syntax on insert and update
    fn support_returning(&self) -> bool {
        let db_backend = self.get_database_backend();
        db_backend.support_returning()
    }

    /// Check if the connection is a test connection for the Mock database
    fn is_mock_connection(&self) -> bool {
        false
    }
}
