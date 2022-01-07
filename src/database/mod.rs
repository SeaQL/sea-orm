#[cfg(feature = "sqlx-any")]
use sqlx::any::AnyKind;
#[cfg(feature = "sqlx-mysql")]
use sqlx::mysql::MySqlConnectOptions;
#[cfg(feature = "sqlx-postgres")]
use sqlx::postgres::PgConnectOptions;
#[cfg(feature = "sqlx-sqlite")]
use sqlx::sqlite::SqliteConnectOptions;
use std::fmt::Debug;
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

use crate::{DbErr, RuntimeErr};

/// Defines a database
#[derive(Debug, Default)]
pub struct Database;

/// Supported database kinds of [sqlx::ConnectOptions]'.
#[derive(Debug, Clone)]
pub enum SqlxConnectOptions {
    #[cfg(feature = "sqlx-mysql")]
    /// Variant for [MySqlConnectOptions]
    MySql(MySqlConnectOptions),
    #[cfg(feature = "sqlx-postgres")]
    /// Variant for [PgConnectOptions]
    Postgres(PgConnectOptions),
    #[cfg(feature = "sqlx-sqlite")]
    /// Variant for [SqliteConnectOptions]
    Sqlite(SqliteConnectOptions),
    #[cfg(feature = "mock")]
    /// Variant for a mock connection
    Mock(DbBackend),
}

impl SqlxConnectOptions {
    /// The database backend type
    pub fn get_db_backend_type(&self) -> DbBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            SqlxConnectOptions::MySql(_) => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            SqlxConnectOptions::Postgres(_) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            SqlxConnectOptions::Sqlite(_) => DbBackend::Sqlite,
            #[cfg(feature = "mock")]
            SqlxConnectOptions::Mock(db_backend) => *db_backend,
        }
    }

    #[cfg(feature = "mock")]
    /// Create a mock database connection options
    pub fn mock(db_backend: DbBackend) -> SqlxConnectOptions {
        Self::Mock(db_backend)
    }

    #[cfg(feature = "mock")]
    /// Is this for mock connection?
    pub fn is_mock(&self) -> bool {
        matches!(self, SqlxConnectOptions::Mock(_))
    }
}

/// Defines the configuration options of a database
#[derive(Debug, Clone)]
pub struct ConnectOptions {
    /// The database sqlx::ConnectOptions used to connect to the database.
    pub(crate) connect_options: SqlxConnectOptions,
    /// The URI of the database, if this struct was created from an URI string, otherwise None
    pub(crate) url: Option<String>,
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
    pub async fn connect<C, E>(opt: C) -> Result<DatabaseConnection, DbErr>
    where
        C: TryInto<ConnectOptions, Error = E> + Debug,
        E: std::error::Error,
    {
        let describe = format!("{:?}", opt);
        let opt: ConnectOptions = opt
            .try_into()
            .map_err(|e| DbErr::Conn(
                RuntimeErr::Internal(format!("Couldn't parse connection options {} {}", describe, e))
            ))?;

        #[cfg(feature = "mock")]
        if opt.connect_options.is_mock() {
            return crate::MockDatabaseConnector::connect(opt).await;
        }

        let backend = opt.connect_options.get_db_backend_type();

        match backend {
            #[cfg(feature = "sqlx-mysql")]
            DbBackend::MySql => crate::SqlxMySqlConnector::connect(opt).await,
            #[cfg(feature = "sqlx-postgres")]
            DbBackend::Postgres => crate::SqlxPostgresConnector::connect(opt).await,
            #[cfg(feature = "sqlx-sqlite")]
            DbBackend::Sqlite => crate::SqlxSqliteConnector::connect(opt).await,
            #[cfg(not(all(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unreachable!(),
        }
    }
}

impl TryFrom<&str> for ConnectOptions {
    type Error = DbErr;

    fn try_from(string: &str) -> Result<Self, Self::Error> {
        ConnectOptions::from_str(string)
    }
}

impl TryFrom<&String> for ConnectOptions {
    type Error = DbErr;

    fn try_from(string: &String) -> Result<Self, Self::Error> {
        ConnectOptions::from_str(string.as_str())
    }
}

impl TryFrom<String> for ConnectOptions {
    type Error = DbErr;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        ConnectOptions::new_from_url(string)
    }
}

#[cfg(feature = "sqlx-mysql")]
impl TryFrom<MySqlConnectOptions> for ConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: MySqlConnectOptions) -> Result<Self, Self::Error> {
        Ok(ConnectOptions::new(SqlxConnectOptions::MySql(
            connect_options,
        )))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl TryFrom<PgConnectOptions> for ConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: PgConnectOptions) -> Result<Self, Self::Error> {
        Ok(ConnectOptions::new(SqlxConnectOptions::Postgres(
            connect_options,
        )))
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl TryFrom<SqliteConnectOptions> for ConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: SqliteConnectOptions) -> Result<Self, Self::Error> {
        Ok(ConnectOptions::new(SqlxConnectOptions::Sqlite(
            connect_options,
        )))
    }
}

#[cfg(feature = "sqlx-any")]
impl TryFrom<sqlx::any::AnyConnectOptions> for ConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: sqlx::any::AnyConnectOptions) -> Result<Self, Self::Error> {
        Ok(ConnectOptions::new(connect_options.try_into()?))
    }
}

#[cfg(feature = "sqlx-mysql")]
impl TryFrom<MySqlConnectOptions> for SqlxConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: MySqlConnectOptions) -> Result<Self, Self::Error> {
        Ok(SqlxConnectOptions::MySql(connect_options))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl TryFrom<PgConnectOptions> for SqlxConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: PgConnectOptions) -> Result<Self, Self::Error> {
        Ok(SqlxConnectOptions::Postgres(connect_options))
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl TryFrom<SqliteConnectOptions> for SqlxConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: SqliteConnectOptions) -> Result<Self, Self::Error> {
        Ok(SqlxConnectOptions::Sqlite(connect_options))
    }
}

#[cfg(feature = "sqlx-any")]
impl TryFrom<sqlx::any::AnyConnectOptions> for SqlxConnectOptions {
    type Error = DbErr;

    fn try_from(connect_options: sqlx::any::AnyConnectOptions) -> Result<Self, Self::Error> {
        match connect_options.kind() {
            #[cfg(feature = "sqlx-postgres")]
            AnyKind::Postgres => Ok(SqlxConnectOptions::Postgres(
                connect_options.as_postgres().unwrap().clone(),
            )),
            #[cfg(feature = "sqlx-mysql")]
            AnyKind::MySql => Ok(SqlxConnectOptions::MySql(
                connect_options.as_mysql().unwrap().clone(),
            )),
            #[cfg(feature = "sqlx-sqlite")]
            AnyKind::Sqlite => Ok(SqlxConnectOptions::Sqlite(
                connect_options.as_sqlite().unwrap().clone(),
            )),
        }
    }
}

impl ConnectOptions {
    /// Create new [ConnectOptions] for a [Database] by passing in a [sqlx::ConnectOptions]
    pub fn new(connect_options: SqlxConnectOptions) -> Self {
        Self {
            connect_options,
            url: None,
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            acquire_timeout: None,
            max_lifetime: None,
            sqlx_logging: true,
            sqlx_logging_level: log::LevelFilter::Info,
            sqlcipher_key: None,
            schema_search_path: None
        }
    }

    /// Create new [ConnectOptions] for a [Database] by passing in a URI string
    pub fn new_from_url(url: String) -> Result<Self, DbErr> {
        Ok(Self {
            connect_options: Self::url_to_sqlx_connect_options(url.clone())?,
            url: Some(url),
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
        })
    }

    fn url_to_sqlx_connect_options(url: String) -> Result<SqlxConnectOptions, DbErr> {
        #[cfg(feature = "sqlx-mysql")]
        if DbBackend::MySql.is_prefix_of(&url) {
            return url
                .parse::<MySqlConnectOptions>()
                .map_err(crate::sqlx_error_to_conn_err)?
                .try_into();
        }
        #[cfg(feature = "sqlx-postgres")]
        if DbBackend::Postgres.is_prefix_of(&url) {
            return url
                .parse::<PgConnectOptions>()
                .map_err(crate::sqlx_error_to_conn_err)?
                .try_into();
        }
        #[cfg(feature = "sqlx-sqlite")]
        if DbBackend::Sqlite.is_prefix_of(&url) {
            return url
                .parse::<SqliteConnectOptions>()
                .map_err(crate::sqlx_error_to_conn_err)?
                .try_into();
        }
        #[cfg(feature = "mock")]
        if crate::MockDatabaseConnector::accepts(&url) {
            if DbBackend::MySql.is_prefix_of(&url) {
                return Ok(SqlxConnectOptions::Mock(DbBackend::MySql));
            }
            #[cfg(feature = "sqlx-postgres")]
            if DbBackend::Postgres.is_prefix_of(&url) {
                return Ok(SqlxConnectOptions::Mock(DbBackend::Postgres));
            }
            #[cfg(feature = "sqlx-sqlite")]
            if DbBackend::Sqlite.is_prefix_of(&url) {
                return Ok(SqlxConnectOptions::Mock(DbBackend::Sqlite));
            }
            return Ok(SqlxConnectOptions::Mock(DbBackend::Postgres));
        }
        Err(DbErr::Conn(RuntimeErr::Internal(format!(
            "The connection string '{}' has no supporting driver.",
            url
        ))))
    }

    fn from_str(url: &str) -> Result<Self, DbErr> {
        Self::new_from_url(url.to_owned())
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

    /// Get the database URL of the pool. This is only present if the pool was created from an URL.
    /// If it was created from some sqlx::ConnectOptions then this method returns None.
    ///
    /// To get the actual ConnectOptions used to connect to the database see: [Self::get_connect_options].
    pub fn get_url(&self) -> &Option<String> {
        &self.url
    }

    /// Get the ConnectOptions used to connect to the database
    pub fn get_connect_options(&self) -> &SqlxConnectOptions {
        &self.connect_options
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
    pub fn set_schema_search_path(&mut self, schema_search_path: String) -> &mut Self {
        self.schema_search_path = Some(schema_search_path);
        self
    }
}
