use sea_orm::{
    AccessMode, ConnectionTrait, DatabaseExecutor, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IntoDatabaseExecutor, IsolationLevel, QueryResult, Schema, SchemaBuilder,
    Statement, TransactionError, TransactionTrait,
};
use std::future::Future;
use std::pin::Pin;

pub struct SchemaManagerConnection<'c>(pub DatabaseExecutor<'c>);

impl<'c> std::ops::Deref for SchemaManagerConnection<'c> {
    type Target = DatabaseExecutor<'c>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for SchemaManagerConnection<'_> {
    fn get_database_backend(&self) -> DbBackend {
        self.0.get_database_backend()
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        self.0.execute_raw(stmt).await
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        self.0.execute_unprepared(sql).await
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        self.0.query_one_raw(stmt).await
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.0.query_all_raw(stmt).await
    }
}

#[async_trait::async_trait]
impl TransactionTrait for SchemaManagerConnection<'_> {
    type Transaction = DatabaseTransaction;

    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        self.0.begin().await
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        self.0.begin_with_config(isolation_level, access_mode).await
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
        self.0.transaction(callback).await
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
        self.0
            .transaction_with_config(callback, isolation_level, access_mode)
            .await
    }
}

#[cfg(feature = "sqlx-dep")]
mod sea_schema_shim {
    use super::SchemaManagerConnection;
    use sea_orm::{DatabaseConnection, DatabaseExecutor, DatabaseTransaction};
    use sea_orm::sea_query::SelectStatement;
    use sea_schema::sqlx_types::{SqlxError, SqlxRow};

    #[async_trait::async_trait]
    impl sea_schema::Connection for SchemaManagerConnection<'_> {
        async fn query_all(&self, select: SelectStatement) -> Result<Vec<SqlxRow>, SqlxError> {
            match &self.0 {
                DatabaseExecutor::Connection(conn) => {
                    <DatabaseConnection as sea_schema::Connection>::query_all(conn, select).await
                }
                DatabaseExecutor::Transaction(txn) => {
                    <DatabaseTransaction as sea_schema::Connection>::query_all(txn, select).await
                }
            }
        }

        async fn query_all_raw(&self, sql: String) -> Result<Vec<SqlxRow>, SqlxError> {
            match &self.0 {
                DatabaseExecutor::Connection(conn) => {
                    <DatabaseConnection as sea_schema::Connection>::query_all_raw(conn, sql).await
                }
                DatabaseExecutor::Transaction(txn) => {
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

impl<'c, T> IntoSchemaManagerConnection<'c> for T
where
    T: IntoDatabaseExecutor<'c> + 'c,
{
    fn into_schema_manager_connection(self) -> SchemaManagerConnection<'c> {
        SchemaManagerConnection(self.into_database_executor())
    }
}
