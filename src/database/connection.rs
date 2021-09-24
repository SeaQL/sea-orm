use crate::{error::*, ExecResult, QueryResult, Statement, StatementBuilder};
use sea_query::{MysqlQueryBuilder, PostgresQueryBuilder, QueryBuilder, SqliteQueryBuilder};

#[cfg_attr(not(feature = "mock"), derive(Clone))]
pub enum DatabaseConnection {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlPoolConnection(crate::SqlxMySqlPoolConnection),
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgresPoolConnection(crate::SqlxPostgresPoolConnection),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlitePoolConnection(crate::SqlxSqlitePoolConnection),
    #[cfg(feature = "mock")]
    MockDatabaseConnection(crate::MockDatabaseConnection),
    Disconnected,
}

pub type DbConn = DatabaseConnection;

pub trait IntoDbBackend {
    fn build<S>(&self, statement: &S) -> Statement
    where
        S: StatementBuilder;

    fn get_query_builder(&self) -> Box<dyn QueryBuilder>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DbBackend {
    MySql,
    Postgres,
    Sqlite,
}

impl IntoDbBackend for DbBackend {
    fn build<S>(&self, statement: &S) -> Statement
    where
        S: StatementBuilder,
    {
        statement.build(self)
    }

    fn get_query_builder(&self) -> Box<dyn QueryBuilder> {
        match self {
            Self::MySql => Box::new(MysqlQueryBuilder),
            Self::Postgres => Box::new(PostgresQueryBuilder),
            Self::Sqlite => Box::new(SqliteQueryBuilder),
        }
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
                #[cfg(feature = "sqlx-postgres")]
                Self::SqlxPostgresPoolConnection(_) => "SqlxPostgresPoolConnection",
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
    pub fn get_database_backend(&self) -> DbBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(_) => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(_) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(_) => DbBackend::Sqlite,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.get_database_backend(),
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }

    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.execute(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.execute(stmt).await,
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_one(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_one(stmt).await,
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
        }
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseConnection::SqlxPostgresPoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseConnection::SqlxSqlitePoolConnection(conn) => conn.query_all(stmt).await,
            #[cfg(feature = "mock")]
            DatabaseConnection::MockDatabaseConnection(conn) => conn.query_all(stmt).await,
            DatabaseConnection::Disconnected => Err(DbErr::Conn("Disconnected".to_owned())),
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

#[cfg(test)]
mod tests {
    use crate::DatabaseConnection;

    #[test]
    fn assert_database_connection_traits() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<DatabaseConnection>();
    }
}
