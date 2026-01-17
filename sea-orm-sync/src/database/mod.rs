use std::{sync::Arc, time::Duration};

#[cfg(feature = "sqlx-mysql")]
use sqlx::mysql::MySqlConnectOptions;
#[cfg(feature = "sqlx-postgres")]
use sqlx::postgres::PgConnectOptions;
#[cfg(feature = "sqlx-sqlite")]
use sqlx::sqlite::SqliteConnectOptions;

mod connection;
mod db_connection;
mod executor;
#[cfg(feature = "mock")]
#[cfg_attr(docsrs, doc(cfg(feature = "mock")))]
mod mock;
#[cfg(feature = "proxy")]
#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
mod proxy;
#[cfg(feature = "rbac")]
mod restricted_connection;
#[cfg(all(feature = "schema-sync", feature = "rusqlite"))]
mod sea_schema_rusqlite;
#[cfg(all(feature = "schema-sync", feature = "sqlx-dep"))]
mod sea_schema_shim;
mod statement;
mod stream;
mod tracing_spans;
mod transaction;

pub use connection::*;
pub use db_connection::*;
pub use executor::*;
#[cfg(feature = "mock")]
#[cfg_attr(docsrs, doc(cfg(feature = "mock")))]
pub use mock::*;
#[cfg(feature = "proxy")]
#[cfg_attr(docsrs, doc(cfg(feature = "proxy")))]
pub use proxy::*;
#[cfg(feature = "rbac")]
pub use restricted_connection::*;
pub use statement::*;
use std::borrow::Cow;
pub use stream::*;
use tracing::instrument;
pub use transaction::*;

use crate::error::*;

/// Defines a database
#[derive(Debug, Default)]
pub struct Database;

#[cfg(feature = "sync")]
type BoxFuture<'a, T> = T;

type AfterConnectCallback =
    Option<Arc<dyn Fn(DatabaseConnection) -> BoxFuture<'static, Result<(), DbErr>> + 'static>>;

/// Defines the configuration options of a database
#[derive(derive_more::Debug, Clone)]
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
    pub(crate) idle_timeout: Option<Option<Duration>>,
    /// Set the maximum amount of time to spend waiting for acquiring a connection
    pub(crate) acquire_timeout: Option<Duration>,
    /// Set the maximum lifetime of individual connections
    pub(crate) max_lifetime: Option<Option<Duration>>,
    /// Enable SQLx statement logging
    pub(crate) sqlx_logging: bool,
    /// SQLx statement logging level (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_logging_level: log::LevelFilter,
    /// SQLx slow statements logging level (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_slow_statements_logging_level: log::LevelFilter,
    /// SQLx slow statements duration threshold (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_slow_statements_logging_threshold: Duration,
    /// set sqlcipher key
    pub(crate) sqlcipher_key: Option<Cow<'static, str>>,
    /// Schema search path (PostgreSQL only)
    pub(crate) schema_search_path: Option<String>,
    pub(crate) test_before_acquire: bool,
    /// Only establish connections to the DB as needed. If set to `true`, the db connection will
    /// be created using SQLx's [connect_lazy](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.connect_lazy)
    /// method.
    pub(crate) connect_lazy: bool,

    #[debug(skip)]
    pub(crate) after_connect: AfterConnectCallback,

    #[cfg(feature = "sqlx-mysql")]
    #[debug(skip)]
    pub(crate) mysql_opts_fn: Option<Arc<dyn Fn(MySqlConnectOptions) -> MySqlConnectOptions>>,
    #[cfg(feature = "sqlx-postgres")]
    #[debug(skip)]
    pub(crate) pg_opts_fn: Option<Arc<dyn Fn(PgConnectOptions) -> PgConnectOptions>>,
    #[cfg(feature = "sqlx-sqlite")]
    #[debug(skip)]
    pub(crate) sqlite_opts_fn: Option<Arc<dyn Fn(SqliteConnectOptions) -> SqliteConnectOptions>>,
}

impl Database {
    /// Method to create a [DatabaseConnection] on a database. This method will return an error
    /// if the database is not available.
    #[instrument(level = "trace", skip(opt))]
    pub fn connect<C>(opt: C) -> Result<DatabaseConnection, DbErr>
    where
        C: Into<ConnectOptions>,
    {
        let opt: ConnectOptions = opt.into();

        if url::Url::parse(&opt.url).is_err() {
            return Err(conn_err(format!(
                "The connection string '{}' cannot be parsed.",
                opt.url
            )));
        }

        #[cfg(feature = "sqlx-mysql")]
        if DbBackend::MySql.is_prefix_of(&opt.url) {
            return crate::SqlxMySqlConnector::connect(opt);
        }
        #[cfg(feature = "sqlx-postgres")]
        if DbBackend::Postgres.is_prefix_of(&opt.url) {
            return crate::SqlxPostgresConnector::connect(opt);
        }
        #[cfg(feature = "sqlx-sqlite")]
        if DbBackend::Sqlite.is_prefix_of(&opt.url) {
            return crate::SqlxSqliteConnector::connect(opt);
        }
        #[cfg(feature = "rusqlite")]
        if DbBackend::Sqlite.is_prefix_of(&opt.url) {
            return crate::driver::rusqlite::RusqliteConnector::connect(opt);
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(&opt.url) {
            return crate::MockDatabaseConnector::connect(&opt.url);
        }

        Err(conn_err(format!(
            "The connection string '{}' has no supporting driver.",
            opt.url
        )))
    }

    /// Method to create a [DatabaseConnection] on a proxy database
    #[cfg(feature = "proxy")]
    #[instrument(level = "trace", skip(proxy_func_arc))]
    pub fn connect_proxy(
        db_type: DbBackend,
        proxy_func_arc: std::sync::Arc<Box<dyn ProxyDatabaseTrait>>,
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
            sqlx_slow_statements_logging_level: log::LevelFilter::Off,
            sqlx_slow_statements_logging_threshold: Duration::from_secs(1),
            sqlcipher_key: None,
            schema_search_path: None,
            test_before_acquire: true,
            connect_lazy: false,
            after_connect: None,
            #[cfg(feature = "sqlx-mysql")]
            mysql_opts_fn: None,
            #[cfg(feature = "sqlx-postgres")]
            pg_opts_fn: None,
            #[cfg(feature = "sqlx-sqlite")]
            sqlite_opts_fn: None,
        }
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

    /// Set the idle duration before closing a connection.
    pub fn idle_timeout<T>(&mut self, value: T) -> &mut Self
    where
        T: Into<Option<Duration>>,
    {
        self.idle_timeout = Some(value.into());
        self
    }

    /// Get the idle duration before closing a connection, if set
    pub fn get_idle_timeout(&self) -> Option<Option<Duration>> {
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

    /// Set the maximum lifetime of individual connections.
    pub fn max_lifetime<T>(&mut self, lifetime: T) -> &mut Self
    where
        T: Into<Option<Duration>>,
    {
        self.max_lifetime = Some(lifetime.into());
        self
    }

    /// Get the maximum lifetime of individual connections, if set
    pub fn get_max_lifetime(&self) -> Option<Option<Duration>> {
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

    /// Set SQLx statement logging level (default INFO).
    /// (ignored if `sqlx_logging` is `false`)
    pub fn sqlx_logging_level(&mut self, level: log::LevelFilter) -> &mut Self {
        self.sqlx_logging_level = level;
        self
    }

    /// Set SQLx slow statements logging level and duration threshold (default `LevelFilter::Off`).
    /// (ignored if `sqlx_logging` is `false`)
    pub fn sqlx_slow_statements_logging_settings(
        &mut self,
        level: log::LevelFilter,
        duration: Duration,
    ) -> &mut Self {
        self.sqlx_slow_statements_logging_level = level;
        self.sqlx_slow_statements_logging_threshold = duration;
        self
    }

    /// Get the level of SQLx statement logging
    pub fn get_sqlx_logging_level(&self) -> log::LevelFilter {
        self.sqlx_logging_level
    }

    /// Get the SQLx slow statements logging settings
    pub fn get_sqlx_slow_statements_logging_settings(&self) -> (log::LevelFilter, Duration) {
        (
            self.sqlx_slow_statements_logging_level,
            self.sqlx_slow_statements_logging_threshold,
        )
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

    /// If true, the connection will be pinged upon acquiring from the pool (default true).
    pub fn test_before_acquire(&mut self, value: bool) -> &mut Self {
        self.test_before_acquire = value;
        self
    }

    /// If set to `true`, the db connection pool will be created using SQLx's
    /// [connect_lazy](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.connect_lazy) method.
    pub fn connect_lazy(&mut self, value: bool) -> &mut Self {
        self.connect_lazy = value;
        self
    }

    /// Get whether DB connections will be established when the pool is created or only as needed.
    pub fn get_connect_lazy(&self) -> bool {
        self.connect_lazy
    }

    /// Set a callback function that will be called after a new connection is established.
    pub fn after_connect<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(DatabaseConnection) -> BoxFuture<'static, Result<(), DbErr>> + 'static,
    {
        self.after_connect = Some(Arc::new(f));

        self
    }

    #[cfg(feature = "sqlx-mysql")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sqlx-mysql")))]
    /// Apply a function to modify the underlying [`MySqlConnectOptions`] before
    /// creating the connection pool.
    pub fn map_sqlx_mysql_opts<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(MySqlConnectOptions) -> MySqlConnectOptions + 'static,
    {
        self.mysql_opts_fn = Some(Arc::new(f));
        self
    }

    #[cfg(feature = "sqlx-postgres")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sqlx-postgres")))]
    /// Apply a function to modify the underlying [`PgConnectOptions`] before
    /// creating the connection pool.
    pub fn map_sqlx_postgres_opts<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(PgConnectOptions) -> PgConnectOptions + 'static,
    {
        self.pg_opts_fn = Some(Arc::new(f));
        self
    }

    #[cfg(feature = "sqlx-sqlite")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sqlx-sqlite")))]
    /// Apply a function to modify the underlying [`SqliteConnectOptions`] before
    /// creating the connection pool.
    pub fn map_sqlx_sqlite_opts<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(SqliteConnectOptions) -> SqliteConnectOptions + 'static,
    {
        self.sqlite_opts_fn = Some(Arc::new(f));
        self
    }
}
