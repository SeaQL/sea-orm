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

use crate::SeaErr;

#[derive(Debug, Default)]
pub struct Database;

impl Database {
    pub async fn connect(string: &str) -> Result<DatabaseConnection, SeaErr> {
        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return Ok(crate::SqlxMySqlConnector::connect(string).await?);
        }
        #[cfg(feature = "sqlx-sqlite")]
        if crate::SqlxSqliteConnector::accepts(string) {
            return Ok(crate::SqlxSqliteConnector::connect(string).await?);
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(string) {
            return Ok(crate::MockDatabaseConnector::connect(string).await?);
        }
        Err(SeaErr::Connection)
    }
}
