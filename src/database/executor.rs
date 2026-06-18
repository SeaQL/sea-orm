use super::transaction::run_async_transaction_callback;
use crate::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Statement, TransactionError, TransactionOptions,
    TransactionTrait,
};
use crate::{Schema, SchemaBuilder};
use std::future::Future;
use std::pin::Pin;

/// Either a borrowed [`DatabaseConnection`] / [`DatabaseTransaction`], or an
/// owned [`DatabaseTransaction`].
///
/// Implements [`ConnectionTrait`] and [`TransactionTrait`], so APIs that
/// need to accept "any of those three" can take a `DatabaseExecutor` and
/// not worry about which variant the caller had. Used in particular by
/// `sea-orm-migration`'s `SchemaManager`.
#[derive(Debug)]
pub enum DatabaseExecutor<'c> {
    /// Borrowed connection — use against a long-lived pool.
    Connection(&'c DatabaseConnection),
    /// Borrowed transaction — caller still owns the transaction handle.
    Transaction(&'c DatabaseTransaction),
    /// Owned transaction — used by migration's `SchemaManager::begin()` so
    /// the transaction can be committed/rolled back at the end of the call.
    OwnedTransaction(DatabaseTransaction),
}

impl<'c> From<&'c DatabaseConnection> for DatabaseExecutor<'c> {
    fn from(conn: &'c DatabaseConnection) -> Self {
        Self::Connection(conn)
    }
}

impl<'c> From<&'c DatabaseTransaction> for DatabaseExecutor<'c> {
    fn from(trans: &'c DatabaseTransaction) -> Self {
        Self::Transaction(trans)
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for DatabaseExecutor<'_> {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            DatabaseExecutor::Connection(conn) => conn.get_database_backend(),
            DatabaseExecutor::Transaction(trans) => trans.get_database_backend(),
            DatabaseExecutor::OwnedTransaction(trans) => trans.get_database_backend(),
        }
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_raw(stmt).await,
            DatabaseExecutor::Transaction(trans) => trans.execute_raw(stmt).await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.execute_raw(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_unprepared(sql).await,
            DatabaseExecutor::Transaction(trans) => trans.execute_unprepared(sql).await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.execute_unprepared(sql).await,
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_one_raw(stmt).await,
            DatabaseExecutor::Transaction(trans) => trans.query_one_raw(stmt).await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.query_one_raw(stmt).await,
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_all_raw(stmt).await,
            DatabaseExecutor::Transaction(trans) => trans.query_all_raw(stmt).await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.query_all_raw(stmt).await,
        }
    }
}

#[async_trait::async_trait]
impl TransactionTrait for DatabaseExecutor<'_> {
    type Transaction = DatabaseTransaction;

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.begin().await,
            DatabaseExecutor::Transaction(trans) => trans.begin().await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.begin().await,
        }
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => {
                conn.begin_with_config(isolation_level, access_mode).await
            }
            DatabaseExecutor::Transaction(trans) => {
                trans.begin_with_config(isolation_level, access_mode).await
            }
            DatabaseExecutor::OwnedTransaction(trans) => {
                trans.begin_with_config(isolation_level, access_mode).await
            }
        }
    }

    async fn begin_with_options(
        &self,
        options: TransactionOptions,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.begin_with_options(options).await,
            DatabaseExecutor::Transaction(trans) => trans.begin_with_options(options).await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.begin_with_options(options).await,
        }
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match self {
            DatabaseExecutor::Connection(conn) => conn.transaction(callback).await,
            DatabaseExecutor::Transaction(trans) => trans.transaction(callback).await,
            DatabaseExecutor::OwnedTransaction(trans) => trans.transaction(callback).await,
        }
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match self {
            DatabaseExecutor::Connection(conn) => {
                conn.transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
            DatabaseExecutor::Transaction(trans) => {
                trans
                    .transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
            DatabaseExecutor::OwnedTransaction(trans) => {
                trans
                    .transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
        }
    }
}

/// Conversion into a [`DatabaseExecutor`]. Implemented for
/// `&DatabaseConnection`, `&DatabaseTransaction`, and owned
/// `DatabaseTransaction` — let users hand any of them to functions that
/// take `impl IntoDatabaseExecutor<'_>`.
pub trait IntoDatabaseExecutor<'c>: Send
where
    Self: 'c,
{
    /// Build the [`DatabaseExecutor`].
    fn into_database_executor(self) -> DatabaseExecutor<'c>;
}

impl<'c> IntoDatabaseExecutor<'c> for DatabaseExecutor<'c> {
    fn into_database_executor(self) -> DatabaseExecutor<'c> {
        self
    }
}

impl<'c> IntoDatabaseExecutor<'c> for &'c DatabaseConnection {
    fn into_database_executor(self) -> DatabaseExecutor<'c> {
        DatabaseExecutor::Connection(self)
    }
}

impl<'c> IntoDatabaseExecutor<'c> for &'c DatabaseTransaction {
    fn into_database_executor(self) -> DatabaseExecutor<'c> {
        DatabaseExecutor::Transaction(self)
    }
}

impl IntoDatabaseExecutor<'static> for DatabaseTransaction {
    fn into_database_executor(self) -> DatabaseExecutor<'static> {
        DatabaseExecutor::OwnedTransaction(self)
    }
}

impl DatabaseExecutor<'_> {
    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back.
    /// Otherwise, the transaction will be committed.
    pub async fn transaction_async<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> AsyncFnOnce(&'c DatabaseTransaction) -> Result<T, E> + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let transaction = self.begin().await.map_err(TransactionError::Connection)?;
        run_async_transaction_callback(transaction, callback).await
    }

    /// Execute the function inside a transaction with isolation level and/or access mode.
    /// If the function returns an error, the transaction will be rolled back.
    /// Otherwise, the transaction will be committed.
    pub async fn transaction_with_config_async<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> AsyncFnOnce(&'c DatabaseTransaction) -> Result<T, E> + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let transaction = self
            .begin_with_config(isolation_level, access_mode)
            .await
            .map_err(TransactionError::Connection)?;
        run_async_transaction_callback(transaction, callback).await
    }

    /// Returns `true` if this executor is backed by a transaction (borrowed or owned).
    pub fn is_transaction(&self) -> bool {
        matches!(
            self,
            DatabaseExecutor::Transaction(_) | DatabaseExecutor::OwnedTransaction(_)
        )
    }

    /// Creates a [`SchemaBuilder`] for this backend
    pub fn get_schema_builder(&self) -> SchemaBuilder {
        Schema::new(self.get_database_backend()).builder()
    }

    #[cfg(feature = "entity-registry")]
    #[cfg_attr(docsrs, doc(cfg(feature = "entity-registry")))]
    /// Builds a schema for all the entities in the given module
    pub fn get_schema_registry(&self, prefix: &str) -> SchemaBuilder {
        let schema = Schema::new(self.get_database_backend());
        crate::EntityRegistry::build_schema(schema, prefix)
    }
}
