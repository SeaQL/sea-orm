use futures_util::lock::Mutex;
use log::LevelFilter;
use sea_query::Values;
use std::{future::Future, pin::Pin, sync::Arc};

use sqlx::{
    Connection, Executor, Sqlite, SqlitePool,
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqliteQueryResult, SqliteRow},
};

use sea_query_sqlx::SqlxValues;
use tracing::{instrument, warn};

use crate::{
    AccessMode, ConnectOptions, DatabaseConnection, DatabaseConnectionType, DatabaseTransaction,
    IsolationLevel, QueryStream, Statement, TransactionError, debug_print, error::*, executor::*,
    sqlx_error_to_exec_err,
};

use super::sqlx_common::*;

/// Defines the [sqlx::sqlite] connector
#[derive(Debug)]
pub struct SqlxSqliteConnector;

/// Defines a sqlx SQLite pool
#[derive(Clone)]
pub struct SqlxSqlitePoolConnection {
    pub(crate) pool: SqlitePool,
    metric_callback: Option<crate::metric::Callback>,
}

impl std::fmt::Debug for SqlxSqlitePoolConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqlxSqlitePoolConnection {{ pool: {:?} }}", self.pool)
    }
}

impl From<SqlitePool> for SqlxSqlitePoolConnection {
    fn from(pool: SqlitePool) -> Self {
        SqlxSqlitePoolConnection {
            pool,
            metric_callback: None,
        }
    }
}

impl From<SqlitePool> for DatabaseConnection {
    fn from(pool: SqlitePool) -> Self {
        DatabaseConnectionType::SqlxSqlitePoolConnection(pool.into()).into()
    }
}

impl SqlxSqliteConnector {
    /// Check if the URI provided corresponds to `sqlite:` for a SQLite database
    pub fn accepts(string: &str) -> bool {
        string.starts_with("sqlite:") && string.parse::<SqliteConnectOptions>().is_ok()
    }

    /// Add configuration options for the SQLite database
    #[instrument(level = "trace")]
    pub async fn connect(options: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
        let mut options = options;
        let mut sqlx_opts = options
            .url
            .parse::<SqliteConnectOptions>()
            .map_err(sqlx_error_to_conn_err)?;
        if let Some(sqlcipher_key) = &options.sqlcipher_key {
            sqlx_opts = sqlx_opts.pragma("key", sqlcipher_key.clone());
        }
        use sqlx::ConnectOptions;
        if !options.sqlx_logging {
            sqlx_opts = sqlx_opts.disable_statement_logging();
        } else {
            sqlx_opts = sqlx_opts.log_statements(options.sqlx_logging_level);
            if options.sqlx_slow_statements_logging_level != LevelFilter::Off {
                sqlx_opts = sqlx_opts.log_slow_statements(
                    options.sqlx_slow_statements_logging_level,
                    options.sqlx_slow_statements_logging_threshold,
                );
            }
        }

        if options.get_max_connections().is_none() {
            options.max_connections(1);
        }

        if let Some(f) = &options.sqlite_opts_fn {
            sqlx_opts = f(sqlx_opts);
        }

        let after_conn = options.after_connect.clone();

        let pool = if options.connect_lazy {
            options.sqlx_pool_options().connect_lazy_with(sqlx_opts)
        } else {
            options
                .sqlx_pool_options()
                .connect_with(sqlx_opts)
                .await
                .map_err(sqlx_error_to_conn_err)?
        };

        let pool = SqlxSqlitePoolConnection {
            pool,
            metric_callback: None,
        };

        #[cfg(feature = "sqlite-use-returning-for-3_35")]
        {
            let version = get_version(&pool).await?;
            ensure_returning_version(&version)?;
        }

        let conn: DatabaseConnection =
            DatabaseConnectionType::SqlxSqlitePoolConnection(pool).into();

        if let Some(cb) = after_conn {
            cb(conn.clone()).await?;
        }

        Ok(conn)
    }
}

impl SqlxSqliteConnector {
    /// Instantiate a sqlx pool connection to a [DatabaseConnection]
    pub fn from_sqlx_sqlite_pool(pool: SqlitePool) -> DatabaseConnection {
        DatabaseConnectionType::SqlxSqlitePoolConnection(SqlxSqlitePoolConnection {
            pool,
            metric_callback: None,
        })
        .into()
    }
}

impl SqlxSqlitePoolConnection {
    /// Execute a [Statement] on a SQLite backend
    #[instrument(level = "trace")]
    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        let mut conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        crate::metric::metric!(self.metric_callback, &stmt, {
            match query.execute(&mut *conn).await {
                Ok(res) => Ok(res.into()),
                Err(err) => Err(sqlx_error_to_exec_err(err)),
            }
        })
    }

    /// Execute an unprepared SQL statement on a SQLite backend
    #[instrument(level = "trace")]
    pub async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        debug_print!("{}", sql);

        let conn = &mut self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        match conn.execute(sql).await {
            Ok(res) => Ok(res.into()),
            Err(err) => Err(sqlx_error_to_exec_err(err)),
        }
    }

    /// Get one result from a SQL query. Returns [Option::None] if no match was found
    #[instrument(level = "trace")]
    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        let mut conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        crate::metric::metric!(self.metric_callback, &stmt, {
            match query.fetch_one(&mut *conn).await {
                Ok(row) => Ok(Some(row.into())),
                Err(err) => match err {
                    sqlx::Error::RowNotFound => Ok(None),
                    _ => Err(sqlx_error_to_query_err(err)),
                },
            }
        })
    }

    /// Get the results of a query returning them as a Vec<[QueryResult]>
    #[instrument(level = "trace")]
    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        let mut conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        crate::metric::metric!(self.metric_callback, &stmt, {
            match query.fetch_all(&mut *conn).await {
                Ok(rows) => Ok(rows.into_iter().map(|r| r.into()).collect()),
                Err(err) => Err(sqlx_error_to_query_err(err)),
            }
        })
    }

    /// Stream the results of executing a SQL query
    #[instrument(level = "trace")]
    pub async fn stream(&self, stmt: Statement) -> Result<QueryStream, DbErr> {
        debug_print!("{}", stmt);

        let conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        Ok(QueryStream::from((
            conn,
            stmt,
            self.metric_callback.clone(),
        )))
    }

    /// Bundle a set of SQL statements that execute together.
    #[instrument(level = "trace")]
    pub async fn begin(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        let conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        DatabaseTransaction::new_sqlite(
            conn,
            self.metric_callback.clone(),
            isolation_level,
            access_mode,
        )
        .await
    }

    /// Create a MySQL transaction
    #[instrument(level = "trace", skip(callback))]
    pub async fn transaction<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(
                &'b DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        let transaction = DatabaseTransaction::new_sqlite(
            conn,
            self.metric_callback.clone(),
            isolation_level,
            access_mode,
        )
        .await
        .map_err(|e| TransactionError::Connection(e))?;
        transaction.run(callback).await
    }

    pub(crate) fn set_metric_callback<F>(&mut self, callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + Send + Sync + 'static,
    {
        self.metric_callback = Some(Arc::new(callback));
    }

    /// Checks if a connection to the database is still valid.
    pub async fn ping(&self) -> Result<(), DbErr> {
        let conn = &mut self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        match conn.ping().await {
            Ok(_) => Ok(()),
            Err(err) => Err(sqlx_error_to_conn_err(err)),
        }
    }

    /// Explicitly close the SQLite connection.
    /// See [`Self::close_by_ref`] for usage with references.
    pub async fn close(self) -> Result<(), DbErr> {
        self.close_by_ref().await
    }

    /// Explicitly close the SQLite connection
    pub async fn close_by_ref(&self) -> Result<(), DbErr> {
        self.pool.close().await;
        Ok(())
    }
}

impl From<SqliteRow> for QueryResult {
    fn from(row: SqliteRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxSqlite(row),
        }
    }
}

impl From<SqliteQueryResult> for ExecResult {
    fn from(result: SqliteQueryResult) -> ExecResult {
        ExecResult {
            result: ExecResultHolder::SqlxSqlite(result),
        }
    }
}

pub(crate) fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, Sqlite, SqlxValues> {
    let values = stmt
        .values
        .as_ref()
        .map_or(Values(Vec::new()), |values| values.clone());
    sqlx::query_with(&stmt.sql, SqlxValues(values))
}

pub(crate) async fn set_transaction_config(
    _conn: &mut PoolConnection<Sqlite>,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
) -> Result<(), DbErr> {
    if isolation_level.is_some() {
        warn!("Setting isolation level in a SQLite transaction isn't supported");
    }
    if access_mode.is_some() {
        warn!("Setting access mode in a SQLite transaction isn't supported");
    }
    Ok(())
}

#[cfg(feature = "sqlite-use-returning-for-3_35")]
async fn get_version(conn: &SqlxSqlitePoolConnection) -> Result<String, DbErr> {
    let stmt = Statement {
        sql: "SELECT sqlite_version()".to_string(),
        values: None,
        db_backend: crate::DbBackend::Sqlite,
    };
    conn.query_one(stmt)
        .await?
        .ok_or_else(|| {
            DbErr::Conn(RuntimeErr::Internal(
                "Error reading SQLite version".to_string(),
            ))
        })?
        .try_get_by(0)
}

#[cfg(feature = "sqlite-use-returning-for-3_35")]
fn ensure_returning_version(version: &str) -> Result<(), DbErr> {
    let mut parts = version.trim().split('.').map(|part| {
        part.parse::<u32>().map_err(|_| {
            DbErr::Conn(RuntimeErr::Internal(
                "Error parsing SQLite version".to_string(),
            ))
        })
    });

    let mut extract_next = || {
        parts.next().transpose().and_then(|part| {
            part.ok_or_else(|| {
                DbErr::Conn(RuntimeErr::Internal("SQLite version too short".to_string()))
            })
        })
    };

    let major = extract_next()?;
    let minor = extract_next()?;

    if major > 3 || (major == 3 && minor >= 35) {
        Ok(())
    } else {
        Err(DbErr::Conn(RuntimeErr::Internal(
            "SQLite version does not support returning".to_string(),
        )))
    }
}

impl
    From<(
        PoolConnection<sqlx::Sqlite>,
        Statement,
        Option<crate::metric::Callback>,
    )> for crate::QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            PoolConnection<sqlx::Sqlite>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        crate::QueryStream::build(stmt, crate::InnerConnection::Sqlite(conn), metric_callback)
    }
}

impl crate::DatabaseTransaction {
    pub(crate) async fn new_sqlite(
        inner: PoolConnection<sqlx::Sqlite>,
        metric_callback: Option<crate::metric::Callback>,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<crate::DatabaseTransaction, DbErr> {
        Self::begin(
            Arc::new(Mutex::new(crate::InnerConnection::Sqlite(inner))),
            crate::DbBackend::Sqlite,
            metric_callback,
            isolation_level,
            access_mode,
        )
        .await
    }
}

#[cfg(feature = "proxy")]
pub(crate) fn from_sqlx_sqlite_row_to_proxy_row(row: &sqlx::sqlite::SqliteRow) -> crate::ProxyRow {
    // https://docs.rs/sqlx-sqlite/0.7.2/src/sqlx_sqlite/type_info.rs.html
    // https://docs.rs/sqlx-sqlite/0.7.2/sqlx_sqlite/types/index.html
    use sea_query::Value;
    use sqlx::{Column, Row, TypeInfo};
    crate::ProxyRow {
        values: row
            .columns()
            .iter()
            .map(|c| {
                (
                    c.name().to_string(),
                    match c.type_info().name() {
                        "BOOLEAN" => {
                            Value::Bool(row.try_get(c.ordinal()).expect("Failed to get boolean"))
                        }

                        "INTEGER" => {
                            Value::Int(row.try_get(c.ordinal()).expect("Failed to get integer"))
                        }

                        "BIGINT" | "INT8" => Value::BigInt(
                            row.try_get(c.ordinal()).expect("Failed to get big integer"),
                        ),

                        "REAL" => {
                            Value::Double(row.try_get(c.ordinal()).expect("Failed to get double"))
                        }

                        "TEXT" => Value::String(
                            row.try_get::<Option<String>, _>(c.ordinal())
                                .expect("Failed to get string")
                                .map(Box::new),
                        ),

                        "BLOB" => Value::Bytes(
                            row.try_get::<Option<Vec<u8>>, _>(c.ordinal())
                                .expect("Failed to get bytes")
                                .map(Box::new),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "DATETIME" => {
                            use chrono::{DateTime, Utc};

                            Value::ChronoDateTimeUtc(
                                row.try_get::<Option<DateTime<Utc>>, _>(c.ordinal())
                                    .expect("Failed to get timestamp")
                                    .map(Box::new),
                            )
                        }
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATETIME" => {
                            use time::OffsetDateTime;
                            Value::TimeDateTimeWithTimeZone(
                                row.try_get::<Option<OffsetDateTime>, _>(c.ordinal())
                                    .expect("Failed to get timestamp")
                                    .map(Box::new),
                            )
                        }
                        #[cfg(feature = "with-chrono")]
                        "DATE" => {
                            use chrono::NaiveDate;
                            Value::ChronoDate(
                                row.try_get::<Option<NaiveDate>, _>(c.ordinal())
                                    .expect("Failed to get date")
                                    .map(Box::new),
                            )
                        }
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATE" => {
                            use time::Date;
                            Value::TimeDate(
                                row.try_get::<Option<Date>, _>(c.ordinal())
                                    .expect("Failed to get date")
                                    .map(Box::new),
                            )
                        }

                        #[cfg(feature = "with-chrono")]
                        "TIME" => {
                            use chrono::NaiveTime;
                            Value::ChronoTime(
                                row.try_get::<Option<NaiveTime>, _>(c.ordinal())
                                    .expect("Failed to get time")
                                    .map(Box::new),
                            )
                        }
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIME" => {
                            use time::Time;
                            Value::TimeTime(
                                row.try_get::<Option<Time>, _>(c.ordinal())
                                    .expect("Failed to get time")
                                    .map(Box::new),
                            )
                        }

                        _ => unreachable!("Unknown column type: {}", c.type_info().name()),
                    },
                )
            })
            .collect(),
    }
}

#[cfg(all(test, feature = "sqlite-use-returning-for-3_35"))]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_returning_version() {
        assert!(ensure_returning_version("").is_err());
        assert!(ensure_returning_version(".").is_err());
        assert!(ensure_returning_version(".a").is_err());
        assert!(ensure_returning_version(".4.9").is_err());
        assert!(ensure_returning_version("a").is_err());
        assert!(ensure_returning_version("1.").is_err());
        assert!(ensure_returning_version("1.a").is_err());

        assert!(ensure_returning_version("1.1").is_err());
        assert!(ensure_returning_version("1.0.").is_err());
        assert!(ensure_returning_version("1.0.0").is_err());
        assert!(ensure_returning_version("2.0.0").is_err());
        assert!(ensure_returning_version("3.34.0").is_err());
        assert!(ensure_returning_version("3.34.999").is_err());

        // valid version
        assert!(ensure_returning_version("3.35.0").is_ok());
        assert!(ensure_returning_version("3.35.1").is_ok());
        assert!(ensure_returning_version("3.36.0").is_ok());
        assert!(ensure_returning_version("4.0.0").is_ok());
        assert!(ensure_returning_version("99.0.0").is_ok());
    }
}
