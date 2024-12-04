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
    /// SQLx slow statements logging level (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_slow_statements_logging_level: log::LevelFilter,
    /// SQLx slow statements duration threshold (ignored if `sqlx_logging` is false)
    pub(crate) sqlx_slow_statements_logging_threshold: Duration,
    /// set sqlcipher key
    pub(crate) sqlcipher_key: Option<Cow<'static, str>>,
    /// Schema search path (PostgreSQL only)
    pub(crate) schema_search_path: Option<String>,
    pub(crate) test_before_acquire: bool,
    /// Sets whether or with what priority a secure SSL TCP/IP connection will be negotiated
    pub(crate) ssl_mode: Option<SSLMode>,
    /// Sets the SSL client certificate as a PEM-encoded byte slice.
    pub(crate) ssl_client_cert: Option<Vec<u8>>,
    /// Sets the SSL client key as a PEM-encoded byte slice
    pub(crate) ssl_client_key: Option<Vec<u8>>,
    /// Sets PEM encoded trusted SSL Certificate Authorities (CA).
    pub(crate) ssl_root_cert: Option<Vec<u8>>,
    /// Only establish connections to the DB as needed. If set to `true`, the db connection will
    /// be created using SQLx's [connect_lazy](https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#method.connect_lazy)
    /// method.
    pub(crate) connect_lazy: bool,
}

impl Database {
    /// Method to create a [DatabaseConnection] on a database. This method will return an error
    /// if the database is not available.
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

#[derive(Debug, Clone, Copy)]
/// Options for controlling the level of protection provided for MySQL or PostgreSQL SSL connections.
pub enum SSLMode {
    /// I don't care about security, and I don't want to pay the overhead of encryption.
    /// This corresponds to postgres `sslmode=disable` and mysql `ssl-mode=DISABLED`.
    Disable,
    /// I don't care about encryption, but I wish to pay the overhead of encryption if the server supports it.
    /// This corresponds to postgres `sslmode=prefer` and mysql `ssl-mode=PREFERRED`.
    /// This is the default.
    Prefer,
    /// I want my data to be encrypted, and I accept the overhead. I trust that the network will make sure I always connect to the server I want.
    /// This corresponds to postgres `sslmode=require` and mysql `ssl-mode=REQUIRED`.
    Require,
    /// I want my data encrypted, and I accept the overhead. I want to be sure that I connect to a server that I trust.
    /// like `Self::Require`, but additionally verify the server Certificate Authority (CA) certificate against the configured CA certificates.
    /// This corresponds to postgres `sslmode=verify-ca` and mysql `ssl-mode=VERIFY_CA`.
    VerifyCa,
    /// I want my data encrypted, and I accept the overhead. I want to be sure that I connect to a server I trust, and that it's the one I specify.
    /// like `Self::VerifyCa`, but additionally perform host name identity verification by checking the host name the client uses for connecting to the server against the identity in the certificate that the server sends to the client.
    /// This corresponds to postgres `sslmode=verify-full` and mysql `ssl-mode=VERIFY_IDENTITY`.
    VerifyIdentity,
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
            ssl_mode: None,
            ssl_client_cert: None,
            ssl_client_key: None,
            ssl_root_cert: None,
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

    /// Sets whether or with what priority a secure SSL TCP/IP connection will be negotiated
    /// with the server.
    ///
    /// By default, the SSL mode is [`Prefer`](SSLMode::Prefer), and the client will
    /// first attempt an SSL connection but fallback to a non-SSL connection on failure.
    ///
    /// Ignored for Unix domain socket communication.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sea_orm::database::{ConnectOptions, SSLMode};
    /// let options = ConnectOptions::new().ssl_mode(SSLMode::Require);
    /// ```
    pub fn ssl_mode(&mut self, mode: SSLMode) -> &mut Self {
        self.ssl_mode = Some(mode);
        self
    }

    /// Sets the SSL client certificate as a PEM-encoded byte slice.
    ///
    /// This should be an ASCII-encoded blob that starts with `-----BEGIN CERTIFICATE-----`.
    ///
    /// # Example
    /// Note: embedding SSL certificates and keys in the binary is not advised.
    /// This is for illustration purposes only.
    ///
    /// ```rust
    /// # use sea_orm::database::{ConnectOptions, SSLMode};
    ///
    /// const CERT: &[u8] = b"\
    /// -----BEGIN CERTIFICATE-----
    /// <Certificate data here.>
    /// -----END CERTIFICATE-----";
    ///
    /// let options = ConnectOptions::new()
    ///     // Providing a CA certificate with less than VerifyCa is pointless
    ///     .ssl_mode(SSLMode::VerifyCa)
    ///     .ssl_client_cert_pem(CERT);
    /// ```
    pub fn ssl_client_cert_pem(&mut self, cert: Vec<u8>) -> &mut Self {
        self.ssl_client_cert = Some(cert);
        self
    }

    /// Sets the SSL client key as a PEM-encoded byte slice.
    ///
    /// This should be an ASCII-encoded blob that starts with `-----BEGIN PRIVATE KEY-----`.
    ///
    /// # Example
    /// Note: embedding SSL certificates and keys in the binary is not advised.
    /// This is for illustration purposes only.
    ///
    /// ```rust
    /// # use sea_orm::database::{ConnectOptions, SSLMode};
    ///
    /// const KEY: &[u8] = b"\
    /// -----BEGIN PRIVATE KEY-----
    /// <Private key data here.>
    /// -----END PRIVATE KEY-----";
    ///
    /// let options = ConnectOptions::new()
    ///     // Providing a CA certificate with less than VerifyCa is pointless
    ///     .ssl_mode(SSLMode::VerifyCa)
    ///     .ssl_client_key_pem(KEY);
    /// ```
    pub fn ssl_client_key_pem(&mut self, key: Vec<u8>) -> &mut Self {
        self.ssl_client_key = Some(key);
        self
    }

    /// Sets PEM encoded trusted SSL Certificate Authorities (CA).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sea_orm::database::{ConnectOptions, SSLMode};
    ///
    /// const CERT: &[u8] = b"\
    /// -----BEGIN CERTIFICATE-----
    /// <Certificate data here.>
    /// -----END CERTIFICATE-----";
    ///
    /// let options = ConnectOptions::new()
    ///     // Providing a CA certificate with less than VerifyCa is pointless
    ///     .ssl_mode(SSLMode::VerifyCa)
    ///     .ssl_root_cert_from_pem(vec![]);
    /// ```
    pub fn ssl_root_cert_from_pem(mut self, pem_certificate: Vec<u8>) -> Self {
        self.ssl_root_cert = Some(pem_certificate);
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
}
