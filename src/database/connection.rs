use crate::{ExecErr, ExecResult, MockDatabaseConnection, QueryErr, QueryResult, Statement};
use sea_query::{
    DeleteStatement, InsertStatement, MysqlQueryBuilder, PostgresQueryBuilder,
    QueryStatementBuilder, SelectStatement, UpdateStatement,
};
use std::{error::Error, fmt};

pub enum DatabaseConnection {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlPoolConnection(crate::SqlxMySqlPoolConnection),
    #[cfg(feature = "mock")]
    MockDatabaseConnection(crate::MockDatabaseConnection),
    Disconnected,
}

pub enum QueryBuilderBackend {
    MySql,
    Postgres,
}

#[derive(Debug)]
pub struct ConnectionErr;

impl Error for ConnectionErr {}

impl fmt::Display for ConnectionErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for DatabaseConnection {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl std::fmt::Debug for DatabaseConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                #[cfg(feature = "sqlx-mysql")]
                Self::SqlxMySqlPoolConnection(_) => "SqlxMySqlPoolConnection",
                #[cfg(feature = "mock")]
                Self::MockDatabaseConnection(_) => "MockDatabaseConnection",
                Self::Disconnected => "Disconnected",
            }
        )
    }
}

impl DatabaseConnection {
    pub fn get_query_builder_backend(&self) -> QueryBuilderBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(_) => QueryBuilderBackend::MySql,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(_) => QueryBuilderBackend::Postgres,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, ExecErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.execute(stmt).await,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, QueryErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_one(stmt).await,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, QueryErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_all(stmt).await,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub fn as_mock_connection(&self) -> &MockDatabaseConnection {
        match self {
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(mock_conn) => mock_conn,
            _ => panic!("not mock connection"),
        }
    }
}

impl QueryBuilderBackend {
    pub fn build<S>(&self, statement: &S) -> Statement
    where
        S: QueryStatementBuilder,
    {
        match self {
            Self::MySql => statement.build(MysqlQueryBuilder),
            Self::Postgres => statement.build(PostgresQueryBuilder),
        }
        .into()
    }
}
