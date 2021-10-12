mod connection;
mod db_connection;
#[cfg(feature = "mock")]
mod mock;
mod statement;
mod stream;
mod transaction;

pub use connection::*;
pub use db_connection::*;
#[cfg(feature = "mock")]
pub use mock::*;
pub use statement::*;
pub use stream::*;
pub use transaction::*;

use crate::DbErr;

#[derive(Debug, Default)]
pub struct Database;

impl Database {
    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        #[cfg(feature = "sqlx-mysql")]
        if DbBackend::MySql.is_prefix_of(string) {
            return crate::SqlxMySqlConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-postgres")]
        if DbBackend::Postgres.is_prefix_of(string) {
            return crate::SqlxPostgresConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if DbBackend::Sqlite.is_prefix_of(string) {
            return crate::SqlxSqliteConnector::connect(string).await;
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(string) {
            return crate::MockDatabaseConnector::connect(string).await;
        }
        Err(DbErr::Conn(format!(
            "The connection string '{}' has no supporting driver.",
            string
        )))
    }
}
