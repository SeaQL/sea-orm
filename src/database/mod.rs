mod connection;
#[cfg(feature = "mock")]
mod mock;
mod statement;
mod transaction;

pub use connection::*;
#[cfg(feature = "mock")]
pub use mock::*;
pub use statement::*;
pub use transaction::*;

use crate::DbErr;

#[derive(Debug, Default)]
pub struct Database;

impl Database {
    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        #[cfg(feature = "sqlx-mysql")]
        if DbBackend::MySql::starts_with(string) {
            return crate::SqlxMySqlConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-postgres")]
        if DbBackend::Postgres::starts_with(string) {
            return crate::SqlxPostgresConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if DbBackend::Sqlite::starts_with(string) {
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
