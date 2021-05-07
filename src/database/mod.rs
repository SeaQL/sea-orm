use crate::{Connection, ConnectionErr, Connector, SqlxMySqlConnector, SqlxMySqlPoolConnection};

#[derive(Debug, Default)]
pub struct Database {
    connection: DatabaseConnection,
}

pub enum DatabaseConnection {
    SqlxMySqlPoolConnection(SqlxMySqlPoolConnection),
    Disconnected,
}

// DatabaseConnection //

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
                Self::SqlxMySqlPoolConnection(_) => "SqlxMySqlPoolConnection",
                Self::Disconnected => "Disconnected",
            }
        )
    }
}

// Database //

impl Database {
    pub async fn connect(&mut self, string: &str) -> Result<(), ConnectionErr> {
        if SqlxMySqlConnector::accepts(string) {
            self.connection = SqlxMySqlConnector::connect(string).await?;
            return Ok(());
        }
        Err(ConnectionErr)
    }

    pub fn get_connection(&self) -> impl Connection + '_ {
        match &self.connection {
            DatabaseConnection::SqlxMySqlPoolConnection(conn) => conn,
            DatabaseConnection::Disconnected => panic!("Disconnected"),
        }
    }
}
