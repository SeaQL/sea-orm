use crate::{error::*, ExecResult, QueryResult, Statement, Syntax};
use sea_query::{
    MysqlQueryBuilder, PostgresQueryBuilder, QueryStatementBuilder, SchemaStatementBuilder,
    SqliteQueryBuilder,
};

pub enum DatabaseConnection {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlPoolConnection(crate::SqlxMySqlPoolConnection),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlitePoolConnection(crate::SqlxSqlitePoolConnection),
    #[cfg(feature = "mock")]
    MockDatabaseConnection(crate::MockDatabaseConnection),
    Disconnected,
}

pub type DbConn = DatabaseConnection;

pub enum QueryBuilderBackend {
    MySql,
    Postgres,
    Sqlite,
}

pub enum SchemaBuilderBackend {
    MySql,
    Postgres,
    Sqlite,
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
                #[cfg(feature = "sqlx-sqlite")]
                Self::SqlxSqlitePoolConnection(_) => "SqlxSqlitePoolConnection",
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
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(_) => QueryBuilderBackend::Sqlite,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.get_query_builder_backend(),
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub fn get_schema_builder_backend(&self) -> SchemaBuilderBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(_) => SchemaBuilderBackend::MySql,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(_) => SchemaBuilderBackend::Sqlite,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.get_schema_builder_backend(),
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.execute(stmt).await,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_one(stmt).await,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_all(stmt).await,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    #[cfg(feature = "mock")]
    pub fn as_mock_connection(&self) -> &crate::MockDatabaseConnection {
        match self {
            DatabaseConnection::MockDatabaseConnection(mock_conn) => mock_conn,
            _ => panic!("not mock connection"),
        }
    }

    #[cfg(not(feature = "mock"))]
    pub fn as_mock_connection(&self) -> Option<bool> {
        None
    }

    #[cfg(feature = "mock")]
    pub fn into_transaction_log(self) -> Vec<crate::Transaction> {
        let mut mocker = self.as_mock_connection().get_mocker_mutex().lock().unwrap();
        mocker.drain_transaction_log()
    }
}

impl QueryBuilderBackend {
    pub fn syntax(&self) -> Syntax {
        match self {
            Self::MySql => Syntax::MySql,
            Self::Postgres => Syntax::Postgres,
            Self::Sqlite => Syntax::Sqlite,
        }
    }

    pub fn build<S>(&self, statement: &S) -> Statement
    where
        S: QueryStatementBuilder,
    {
        Statement::from_string_values_tuple(
            self.syntax(),
            match self {
                Self::MySql => statement.build(MysqlQueryBuilder),
                Self::Postgres => statement.build(PostgresQueryBuilder),
                Self::Sqlite => statement.build(SqliteQueryBuilder),
            },
        )
    }
}

impl SchemaBuilderBackend {
    pub fn syntax(&self) -> Syntax {
        match self {
            Self::MySql => Syntax::MySql,
            Self::Postgres => Syntax::Postgres,
            Self::Sqlite => Syntax::Sqlite,
        }
    }

    pub fn build<S>(&self, statement: &S) -> Statement
    where
        S: SchemaStatementBuilder,
    {
        Statement::from_string(
            self.syntax(),
            match self {
                Self::MySql => statement.build(MysqlQueryBuilder),
                Self::Postgres => statement.build(PostgresQueryBuilder),
                Self::Sqlite => statement.build(SqliteQueryBuilder),
            },
        )
    }
}
