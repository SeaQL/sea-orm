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

#[derive(Debug)]
pub enum DbScheme {
    Postgres,
    Mysql,
    Sqlite,
}

impl DbScheme {
    pub fn starts_with(self, base_url: &str) -> bool {
        match self {
            DbScheme::Postgres => {
                base_url.starts_with("postgres://") || base_url.starts_with("postgresql://")
            }
            DbScheme::Mysql => base_url.starts_with("mysql://"),
            DbScheme::Sqlite => base_url.starts_with("sqlite:"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Database;

impl Database {
    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        #[cfg(feature = "sqlx-mysql")]
        if DbScheme::Mysql::starts_with(string) {
            return crate::SqlxMySqlConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-postgres")]
        if DbScheme::Postgres::starts_with(string) {
            return crate::SqlxPostgresConnector::connect(string).await;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if DbScheme::Sqlite::starts_with(string) {
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
