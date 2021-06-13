mod connection;
#[cfg(feature = "mock")]
mod mock;
mod statement;

pub use connection::*;
#[cfg(feature = "mock")]
pub use mock::*;
pub use statement::*;

#[derive(Debug, Default)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub async fn connect(&mut self, string: &str) -> Result<(), ConnectionErr> {
        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            self.connection = crate::SqlxMySqlConnector::connect(string).await?;
            return Ok(());
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(string) {
            self.connection = crate::MockDatabaseConnector::connect(string).await?;
            return Ok(());
        }
        Err(ConnectionErr)
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn get_query_builder_backend(&self) -> QueryBuilderBackend {
        self.connection.get_query_builder_backend()
    }
}
