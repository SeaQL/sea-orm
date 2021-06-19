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

#[derive(Debug, Default)]
pub struct Database;

impl Database {
    pub async fn connect(string: &str) -> Result<DatabaseConnection, ConnectionErr> {
        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return Ok(crate::SqlxMySqlConnector::connect(string).await?);
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(string) {
            return Ok(crate::MockDatabaseConnector::connect(string).await?);
        }
        Err(ConnectionErr)
    }
}
