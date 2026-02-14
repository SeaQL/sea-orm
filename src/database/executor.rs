use crate::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Statement, TransactionError, TransactionOptions,
    TransactionTrait,
};
use crate::{Schema, SchemaBuilder};
use std::future::Future;
use std::pin::Pin;

/// A wrapper that holds either a reference to a [`DatabaseConnection`] or [`DatabaseTransaction`].
#[derive(Debug)]
pub enum DatabaseExecutor<'c> {
    /// A reference to a database connection
    Connection(&'c DatabaseConnection),
    /// A reference to a database transaction
    Transaction(&'c DatabaseTransaction),
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
        }
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_raw(stmt).await,
            DatabaseExecutor::Transaction(trans) => trans.execute_raw(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_unprepared(sql).await,
            DatabaseExecutor::Transaction(trans) => trans.execute_unprepared(sql).await,
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_one_raw(stmt).await,
            DatabaseExecutor::Transaction(trans) => trans.query_one_raw(stmt).await,
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_all_raw(stmt).await,
            DatabaseExecutor::Transaction(trans) => trans.query_all_raw(stmt).await,
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
        }
    }

    async fn begin_with_options(
        &self,
        options: TransactionOptions,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.begin_with_options(options).await,
            DatabaseExecutor::Transaction(trans) => trans.begin_with_options(options).await,
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
        }
    }
}

/// A trait for converting into [`DatabaseExecutor`]
pub trait IntoDatabaseExecutor<'c>: Send
where
    Self: 'c,
{
    /// Convert into a [`DatabaseExecutor`]
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

impl DatabaseExecutor<'_> {
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
