use std::time::Duration;

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

/// Defines a database
#[derive(Debug, Default)]
pub struct Database;

/// Defines the configuration options of a database
#[derive(Debug)]
pub struct ConnectOptions {
    /// The URI of the database
    pub(crate) url: String,
    /// Maximum number of connections for a pool
    pub(crate) max_connections: Option<u32>,
    /// Minimum number of connections for a pool
    pub(crate) min_connections: Option<u32>,
    /// The connection timeout for a packet connection
    pub(crate) connect_timeout: Option<Duration>,
    /// Maximum idle time for a particular connection to prevent
    /// network resource exhaustion
    pub(crate) idle_timeout: Option<Duration>,
    /// Enables or disables logging
    pub(crate) sqlx_logging: bool,
}

impl Database {
    /// Method to create a [DatabaseConnection] on a database
    pub async fn connect<C>(opt: C) -> Result<DatabaseConnection, DbErr>
    where
        C: Into<ConnectOptions>,
    {
        let opt: ConnectOptions = opt.into();

        #[cfg(feature = "sqlx-mysql")]
        if DbBackend::MySql.is_prefix_of(&opt.url) {
            return crate::SqlxMySqlConnector::connect(opt).await;
        }
        #[cfg(feature = "sqlx-postgres")]
        if DbBackend::Postgres.is_prefix_of(&opt.url) {
            return crate::SqlxPostgresConnector::connect(opt).await;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if DbBackend::Sqlite.is_prefix_of(&opt.url) {
            return crate::SqlxSqliteConnector::connect(opt).await;
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(&opt.url) {
            return crate::MockDatabaseConnector::connect(&opt.url).await;
        }
        Err(DbErr::Conn(format!(
            "The connection string '{}' has no supporting driver.",
            opt.url
        )))
    }
}

impl From<&str> for ConnectOptions {
    fn from(string: &str) -> ConnectOptions {
        ConnectOptions::from_str(string)
    }
}

impl From<&String> for ConnectOptions {
    fn from(string: &String) -> ConnectOptions {
        ConnectOptions::from_str(string.as_str())
    }
}

impl From<String> for ConnectOptions {
    fn from(string: String) -> ConnectOptions {
        ConnectOptions::new(string)
    }
}

impl ConnectOptions {
    /// Create new [ConnectOptions] for a [Database] by passing in a URI string
    pub fn new(url: String) -> Self {
        Self {
            url,
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            sqlx_logging: true,
        }
    }

    fn from_str(url: &str) -> Self {
        Self::new(url.to_owned())
    }

    #[cfg(feature = "sqlx-dep")]
    /// Convert [ConnectOptions] into [sqlx::pool::PoolOptions]
    pub fn pool_options<DB>(self) -> sqlx::pool::PoolOptions<DB>
    where
        DB: sqlx::Database,
    {
        let mut opt = sqlx::pool::PoolOptions::new();
        if let Some(max_connections) = self.max_connections {
            opt = opt.max_connections(max_connections);
        }
        if let Some(min_connections) = self.min_connections {
            opt = opt.min_connections(min_connections);
        }
        if let Some(connect_timeout) = self.connect_timeout {
            opt = opt.connect_timeout(connect_timeout);
        }
        if let Some(idle_timeout) = self.idle_timeout {
            opt = opt.idle_timeout(Some(idle_timeout));
        }
        opt
    }

    /// Set the maximum number of connections of the pool
    pub fn max_connections(&mut self, value: u32) -> &mut Self {
        self.max_connections = Some(value);
        self
    }

    /// Get the maximum number of connections of the pool, if set
    pub fn get_max_connections(&self) -> Option<u32> {
        self.max_connections
    }

    /// Set the minimum number of connections of the pool
    pub fn min_connections(&mut self, value: u32) -> &mut Self {
        self.min_connections = Some(value);
        self
    }

    /// Get the minimum number of connections of the pool, if set
    pub fn get_min_connections(&self) -> Option<u32> {
        self.min_connections
    }

    /// Set the timeout duration when acquiring a connection
    pub fn connect_timeout(&mut self, value: Duration) -> &mut Self {
        self.connect_timeout = Some(value);
        self
    }

    /// Get the timeout duration when acquiring a connection, if set
    pub fn get_connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// Set the idle duration before closing a connection
    pub fn idle_timeout(&mut self, value: Duration) -> &mut Self {
        self.idle_timeout = Some(value);
        self
    }

    /// Get the idle duration before closing a connection, if set
    pub fn get_idle_timeout(&self) -> Option<Duration> {
        self.idle_timeout
    }
}
