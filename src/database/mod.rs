mod connection;
#[cfg(feature = "mock")]
mod mock;
mod statement;
mod transaction;
mod db_connection;
mod db_transaction;
#[cfg(feature = "sqlx-dep")]
mod stream;

pub use connection::*;
#[cfg(feature = "mock")]
pub use mock::*;
pub use statement::*;
pub use transaction::*;
pub use db_connection::*;
pub use db_transaction::*;
#[cfg(feature = "sqlx-dep")]
pub use stream::*;

use crate::DbErr;

#[derive(Debug, Default)]
pub struct Database;

impl Database {
    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return crate::SqlxMySqlConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-postgres")]
        if crate::SqlxPostgresConnector::accepts(string) {
            return crate::SqlxPostgresConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if crate::SqlxSqliteConnector::accepts(string) {
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
