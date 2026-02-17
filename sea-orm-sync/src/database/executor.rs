use crate::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Statement, TransactionError, TransactionTrait,
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

impl ConnectionTrait for DatabaseExecutor<'_> {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            DatabaseExecutor::Connection(conn) => conn.get_database_backend(),
            DatabaseExecutor::Transaction(trans) => trans.get_database_backend(),
        }
    }

    fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_raw(stmt),
            DatabaseExecutor::Transaction(trans) => trans.execute_raw(stmt),
        }
    }

    fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.execute_unprepared(sql),
            DatabaseExecutor::Transaction(trans) => trans.execute_unprepared(sql),
        }
    }

    fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_one_raw(stmt),
            DatabaseExecutor::Transaction(trans) => trans.query_one_raw(stmt),
        }
    }

    fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.query_all_raw(stmt),
            DatabaseExecutor::Transaction(trans) => trans.query_all_raw(stmt),
        }
    }
}

impl TransactionTrait for DatabaseExecutor<'_> {
    type Transaction = DatabaseTransaction;

    fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match self {
            DatabaseExecutor::Connection(conn) => conn.begin(),
            DatabaseExecutor::Transaction(trans) => trans.begin(),
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
        }
    }
}

/// A trait for converting into [`DatabaseExecutor`]
pub trait IntoDatabaseExecutor<'c>
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
        crate::EntityRegistry::build_schema(schema, prefix, None)
    }

    #[cfg(feature = "entity-registry")]
    #[cfg_attr(docsrs, doc(cfg(feature = "entity-registry")))]
    /// Builds a schema for all the entities in the given module and crate version
    pub fn get_schema_registry_version(&self, prefix: &str, version_spec: &str) -> SchemaBuilder {
        let schema = Schema::new(self.get_database_backend());
        crate::EntityRegistry::build_schema(schema, prefix, Some(version_spec))
    }
}
