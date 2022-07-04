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
use std::borrow::Cow;
pub use stream::*;
use tracing::instrument;
pub use transaction::*;

use crate::DbErr;

/// Defines a database
#[derive(Debug, Default)]
pub struct Database;

/// Defines the configuration options of a database
#[derive(Debug, Clone)]
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
    /// Set the maximum lifetime of individual connections
    pub(crate) max_lifetime: Option<Duration>,
    /// Enable SQLx statement logging
    pub(crate) sqlx_logging: bool,
    /// SQLx statement logging level (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_logging_level: log::LevelFilter,
    /// set sqlcipher key
    pub(crate) sqlcipher_key: Option<Cow<'static, str>>,
}

impl Database {
    /// Method to create a [DatabaseConnection] on a database
    #[instrument(level = "trace", skip(opt))]
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
            max_lifetime: None,
            sqlx_logging: true,
            sqlx_logging_level: log::LevelFilter::Info,
            sqlcipher_key: None,
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
            opt = opt.acquire_timeout(connect_timeout);
        }
        if let Some(idle_timeout) = self.idle_timeout {
            opt = opt.idle_timeout(Some(idle_timeout));
        }
        if let Some(max_lifetime) = self.max_lifetime {
            opt = opt.max_lifetime(Some(max_lifetime));
        }
        opt
    }

    /// Get the database URL of the pool
    pub fn get_url(&self) -> &str {
        &self.url
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

    /// Set the maximum lifetime of individual connections
    pub fn max_lifetime(&mut self, lifetime: Duration) -> &mut Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    /// Get the maximum lifetime of individual connections, if set
    pub fn get_max_lifetime(&self) -> Option<Duration> {
        self.max_lifetime
    }

    /// Enable SQLx statement logging (default true)
    pub fn sqlx_logging(&mut self, value: bool) -> &mut Self {
        self.sqlx_logging = value;
        self
    }

    /// Get whether SQLx statement logging is enabled
    pub fn get_sqlx_logging(&self) -> bool {
        self.sqlx_logging
    }

    /// Set SQLx statement logging level (default INFO)
    /// (ignored if `sqlx_logging` is `false`)
    pub fn sqlx_logging_level(&mut self, level: log::LevelFilter) -> &mut Self {
        self.sqlx_logging_level = level;
        self
    }

    /// Get the level of SQLx statement logging
    pub fn get_sqlx_logging_level(&self) -> log::LevelFilter {
        self.sqlx_logging_level
    }

    /// set key for sqlcipher
    pub fn sqlcipher_key<T>(&mut self, value: T) -> &mut Self
    where
        T: Into<Cow<'static, str>>,
    {
        self.sqlcipher_key = Some(value.into());
        self
    }
}
