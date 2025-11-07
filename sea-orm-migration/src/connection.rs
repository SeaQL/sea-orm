use sea_orm::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Schema, SchemaBuilder, Statement, TransactionError,
    TransactionTrait,
};
use std::future::Future;
use std::pin::Pin;

pub enum SchemaManagerConnection<'c> {
    Connection(&'c DatabaseConnection),
    Transaction(&'c DatabaseTransaction),
}

#[async_trait::async_trait]
impl ConnectionTrait for SchemaManagerConnection<'_> {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.get_database_backend(),
            SchemaManagerConnection::Transaction(trans) => trans.get_database_backend(),
        }
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.execute_raw(stmt).await,
            SchemaManagerConnection::Transaction(trans) => trans.execute_raw(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.execute_unprepared(sql).await,
            SchemaManagerConnection::Transaction(trans) => trans.execute_unprepared(sql).await,
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.query_one_raw(stmt).await,
            SchemaManagerConnection::Transaction(trans) => trans.query_one_raw(stmt).await,
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.query_all_raw(stmt).await,
            SchemaManagerConnection::Transaction(trans) => trans.query_all_raw(stmt).await,
        }
    }
}

#[async_trait::async_trait]
impl TransactionTrait for SchemaManagerConnection<'_> {
    type Transaction = DatabaseTransaction;

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.begin().await,
            SchemaManagerConnection::Transaction(trans) => trans.begin().await,
        }
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => {
                conn.begin_with_config(isolation_level, access_mode).await
            }
            SchemaManagerConnection::Transaction(trans) => {
                trans.begin_with_config(isolation_level, access_mode).await
            }
        }
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'a> FnOnce(
                &'a DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.transaction(callback).await,
            SchemaManagerConnection::Transaction(trans) => trans.transaction(callback).await,
        }
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'a> FnOnce(
                &'a DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'a>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match self {
            SchemaManagerConnection::Connection(conn) => {
                conn.transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
            SchemaManagerConnection::Transaction(trans) => {
                trans
                    .transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
        }
    }
}

#[cfg(feature = "sqlx-dep")]
mod sea_schema_shim {
    use super::{DatabaseConnection, DatabaseTransaction, SchemaManagerConnection};
    use sea_orm::sea_query::SelectStatement;
    use sea_schema::sqlx_types::{SqlxError, SqlxRow};

    #[async_trait::async_trait]
    impl sea_schema::Connection for SchemaManagerConnection<'_> {
        async fn query_all(&self, select: SelectStatement) -> Result<Vec<SqlxRow>, SqlxError> {
            match self {
                Self::Connection(conn) => {
                    <DatabaseConnection as sea_schema::Connection>::query_all(conn, select).await
                }
                Self::Transaction(txn) => {
                    <DatabaseTransaction as sea_schema::Connection>::query_all(txn, select).await
                }
            }
        }

        async fn query_all_raw(&self, sql: String) -> Result<Vec<SqlxRow>, SqlxError> {
            match self {
                Self::Connection(conn) => {
                    <DatabaseConnection as sea_schema::Connection>::query_all_raw(conn, sql).await
                }
                Self::Transaction(txn) => {
                    <DatabaseTransaction as sea_schema::Connection>::query_all_raw(txn, sql).await
                }
            }
        }
    }
}

impl SchemaManagerConnection<'_> {
    /// Creates a [`SchemaBuilder`] for this backend
    pub fn get_schema_builder(&self) -> SchemaBuilder {
        Schema::new(self.get_database_backend()).builder()
    }

    #[cfg(feature = "entity-registry")]
    #[cfg_attr(docsrs, doc(cfg(feature = "entity-registry")))]
    /// Builds a schema for all the entites in the given module
    pub fn get_schema_registry(&self, prefix: &str) -> SchemaBuilder {
        let schema = Schema::new(self.get_database_backend());
        sea_orm::EntityRegistry::build_schema(schema, prefix)
    }
}

pub trait IntoSchemaManagerConnection<'c>: Send
where
    Self: 'c,
{
    fn into_schema_manager_connection(self) -> SchemaManagerConnection<'c>;
}

impl<'c> IntoSchemaManagerConnection<'c> for SchemaManagerConnection<'c> {
    fn into_schema_manager_connection(self) -> SchemaManagerConnection<'c> {
        self
    }
}

impl<'c> IntoSchemaManagerConnection<'c> for &'c DatabaseConnection {
    fn into_schema_manager_connection(self) -> SchemaManagerConnection<'c> {
        SchemaManagerConnection::Connection(self)
    }
}

impl<'c> IntoSchemaManagerConnection<'c> for &'c DatabaseTransaction {
    fn into_schema_manager_connection(self) -> SchemaManagerConnection<'c> {
        SchemaManagerConnection::Transaction(self)
    }
}
