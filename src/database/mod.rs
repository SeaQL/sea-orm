use std::time::Duration;

mod connection;
mod db_connection;
#[cfg(feature = "mock")]
#[cfg_attr(docsrs, doc(cfg(feature = "mock")))]
mod mock;
#[cfg(feature = "proxy")]
#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
mod proxy;
mod statement;
mod stream;
mod transaction;

pub use connection::*;
pub use db_connection::*;
#[cfg(feature = "mock")]
#[cfg_attr(docsrs, doc(cfg(feature = "mock")))]
pub use mock::*;
#[cfg(feature = "proxy")]
#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
pub use proxy::*;
pub use statement::*;
use std::borrow::Cow;
pub use stream::*;
use tracing::instrument;
pub use transaction::*;

use crate::error::*;

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
    /// Set the maximum amount of time to spend waiting for acquiring a connection
    pub(crate) acquire_timeout: Option<Duration>,
    /// Set the maximum lifetime of individual connections
    pub(crate) max_lifetime: Option<Duration>,
    /// Enable SQLx statement logging
    pub(crate) sqlx_logging: bool,
    /// SQLx statement logging level (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_logging_level: log::LevelFilter,
    /// set sqlcipher key
    pub(crate) sqlcipher_key: Option<Cow<'static, str>>,
    /// Schema search path (PostgreSQL only)
    pub(crate) schema_search_path: Option<String>,
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
        if crate::driver::MockDatabaseConnector::accepts(&opt.url) {
            return crate::driver::MockDatabaseConnector::connect(&opt.url).await;
        }

        Err(conn_err(format!(
            "The connection string '{}' has no supporting driver.",
            opt.url
        )))
    }

    /// Method to create a [DatabaseConnection] on a proxy database
    #[cfg(feature = "proxy")]
    #[instrument(level = "trace", skip(proxy_func_arc))]
    pub async fn connect_proxy(
        db_type: DbBackend,
        proxy_func_arc: std::sync::Arc<std::sync::Mutex<Box<dyn ProxyDatabaseTrait>>>,
    ) -> Result<DatabaseConnection, DbErr> {
        match db_type {
            DbBackend::MySql => {
                return crate::ProxyDatabaseConnector::connect(
                    DbBackend::MySql,
                    proxy_func_arc.to_owned(),
                );
            }
            DbBackend::Postgres => {
                return crate::ProxyDatabaseConnector::connect(
                    DbBackend::Postgres,
                    proxy_func_arc.to_owned(),
                );
            }
            DbBackend::Sqlite => {
                return crate::ProxyDatabaseConnector::connect(
                    DbBackend::Sqlite,
                    proxy_func_arc.to_owned(),
                );
            }
        }
    }
}

impl<T> From<T> for ConnectOptions
where
    T: Into<String>,
{
    fn from(s: T) -> ConnectOptions {
        ConnectOptions::new(s.into())
    }
}

impl ConnectOptions {
    /// Create new [ConnectOptions] for a [Database] by passing in a URI string
    pub fn new<T>(url: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            url: url.into(),
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            acquire_timeout: None,
            max_lifetime: None,
            sqlx_logging: true,
            sqlx_logging_level: log::LevelFilter::Info,
            sqlcipher_key: None,
            schema_search_path: None,
        }
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
        if let Some(acquire_timeout) = self.acquire_timeout {
            opt = opt.acquire_timeout(acquire_timeout);
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

    /// Set the maximum amount of time to spend waiting for acquiring a connection
    pub fn acquire_timeout(&mut self, value: Duration) -> &mut Self {
        self.acquire_timeout = Some(value);
        self
    }

    /// Get the maximum amount of time to spend waiting for acquiring a connection
    pub fn get_acquire_timeout(&self) -> Option<Duration> {
        self.acquire_timeout
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

    /// Set schema search path (PostgreSQL only)
    pub fn set_schema_search_path<T>(&mut self, schema_search_path: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.schema_search_path = Some(schema_search_path.into());
        self
    }
}
