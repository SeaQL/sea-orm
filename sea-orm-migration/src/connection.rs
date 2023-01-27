use futures::Future;
use sea_orm::{
    AccessMode, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    ExecResult, IsolationLevel, QueryResult, Statement, TransactionError, TransactionTrait,
};
use std::pin::Pin;

pub enum SchemaManagerConnection<'c> {
    Connection(&'c DatabaseConnection),
    Transaction(&'c DatabaseTransaction),
}

#[async_trait::async_trait]
impl<'c> ConnectionTrait for SchemaManagerConnection<'c> {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.get_database_backend(),
            SchemaManagerConnection::Transaction(trans) => trans.get_database_backend(),
        }
    }

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.execute(stmt).await,
            SchemaManagerConnection::Transaction(trans) => trans.execute(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.execute_unprepared(sql).await,
            SchemaManagerConnection::Transaction(trans) => trans.execute_unprepared(sql).await,
        }
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.query_one(stmt).await,
            SchemaManagerConnection::Transaction(trans) => trans.query_one(stmt).await,
        }
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.query_all(stmt).await,
            SchemaManagerConnection::Transaction(trans) => trans.query_all(stmt).await,
        }
    }

    fn is_mock_connection(&self) -> bool {
        match self {
            SchemaManagerConnection::Connection(conn) => conn.is_mock_connection(),
            SchemaManagerConnection::Transaction(trans) => trans.is_mock_connection(),
        }
    }
}

#[async_trait::async_trait]
impl<'c> TransactionTrait for SchemaManagerConnection<'c> {
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
        E: std::error::Error + Send,
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
        E: std::error::Error + Send,
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
