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

impl ConnectionTrait for DatabaseExecutor<'_> {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            DatabaseExecutor::Connection(conn) => conn.get_database_backend(),
            DatabaseExecutor::Transaction(trans) => trans.get_database_backend(),
            DatabaseExecutor::OwnedTransaction(trans) => trans.get_database_backend(),
        }
    }

    fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_raw(stmt),
            DatabaseExecutor::Transaction(trans) => trans.execute_raw(stmt),
            DatabaseExecutor::OwnedTransaction(trans) => trans.execute_raw(stmt),
        }
    }

    fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_unprepared(sql),
            DatabaseExecutor::Transaction(trans) => trans.execute_unprepared(sql),
            DatabaseExecutor::OwnedTransaction(trans) => trans.execute_unprepared(sql),
        }
    }

    fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_one_raw(stmt),
            DatabaseExecutor::Transaction(trans) => trans.query_one_raw(stmt),
            DatabaseExecutor::OwnedTransaction(trans) => trans.query_one_raw(stmt),
        }
    }

    fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_all_raw(stmt),
            DatabaseExecutor::Transaction(trans) => trans.query_all_raw(stmt),
            DatabaseExecutor::OwnedTransaction(trans) => trans.query_all_raw(stmt),
        }
    }
}

impl TransactionTrait for DatabaseExecutor<'_> {
    type Transaction = DatabaseTransaction;

    fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.begin(),
            DatabaseExecutor::Transaction(trans) => trans.begin(),
            DatabaseExecutor::OwnedTransaction(trans) => trans.begin(),
        }
    }

    fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => {
                conn.begin_with_config(isolation_level, access_mode)
            }
            DatabaseExecutor::Transaction(trans) => {
                trans.begin_with_config(isolation_level, access_mode)
            }
            DatabaseExecutor::OwnedTransaction(trans) => {
                trans.begin_with_config(isolation_level, access_mode)
            }
        }
    }

    fn begin_with_options(
        &self,
        options: TransactionOptions,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.begin_with_options(options),
            DatabaseExecutor::Transaction(trans) => trans.begin_with_options(options),
            DatabaseExecutor::OwnedTransaction(trans) => trans.begin_with_options(options),
        }
    }

    fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        match self {
            DatabaseExecutor::Connection(conn) => conn.transaction(callback),
            DatabaseExecutor::Transaction(trans) => trans.transaction(callback),
            DatabaseExecutor::OwnedTransaction(trans) => trans.transaction(callback),
        }
    }

    fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        match self {
            DatabaseExecutor::Connection(conn) => {
                conn.transaction_with_config(callback, isolation_level, access_mode)
            }
            DatabaseExecutor::Transaction(trans) => {
                trans.transaction_with_config(callback, isolation_level, access_mode)
            }
            DatabaseExecutor::OwnedTransaction(trans) => {
                trans.transaction_with_config(callback, isolation_level, access_mode)
            }
        }
    }
}

/// Conversion into a [`DatabaseExecutor`]. Implemented for
/// `&DatabaseConnection`, `&DatabaseTransaction`, and owned
/// `DatabaseTransaction` — let users hand any of them to functions that
/// take `impl IntoDatabaseExecutor<'_>`.
pub trait IntoDatabaseExecutor<'c>
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
    pub fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let transaction = self.begin().map_err(TransactionError::Connection)?;
        run_async_transaction_callback(transaction, callback)
    }

    /// Execute the function inside a transaction with isolation level and/or access mode.
    /// If the function returns an error, the transaction will be rolled back.
    /// Otherwise, the transaction will be committed.
    pub fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        let transaction = self
            .begin_with_config(isolation_level, access_mode)
            .map_err(TransactionError::Connection)?;
        run_async_transaction_callback(transaction, callback)
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
