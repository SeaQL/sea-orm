use sea_query::Values;
use std::{future::Future, pin::Pin, sync::Arc};

use sqlx::{
    mysql::{MySqlConnectOptions, MySqlQueryResult, MySqlRow},
    pool::PoolConnection,
    Connection, Executor, MySql, MySqlPool,
};

use sea_query_binder::SqlxValues;
use tracing::instrument;

use crate::{
    debug_print, error::*, executor::*, AccessMode, ConnectOptions, DatabaseConnection,
    DatabaseTransaction, DbBackend, IsolationLevel, QueryStream, Statement, TransactionError,
};

use super::sqlx_common::*;

/// Defines the [sqlx::mysql] connector
#[derive(Debug)]
pub struct SqlxMySqlConnector;

/// Defines a sqlx MySQL pool
#[derive(Clone)]
pub struct SqlxMySqlPoolConnection {
    pub(crate) pool: MySqlPool,
    metric_callback: Option<crate::metric::Callback>,
}

impl std::fmt::Debug for SqlxMySqlPoolConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqlxMySqlPoolConnection {{ pool: {:?} }}", self.pool)
    }
}

impl SqlxMySqlConnector {
    /// Check if the URI provided corresponds to `mysql://` for a MySQL database
    pub fn accepts(string: &str) -> bool {
        string.starts_with("mysql://") && string.parse::<MySqlConnectOptions>().is_ok()
    }

    /// Add configuration options for the MySQL database
    #[instrument(level = "trace")]
    pub async fn connect(options: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
        let mut opt = options
            .url
            .parse::<MySqlConnectOptions>()
            .map_err(sqlx_error_to_conn_err)?;
        use sqlx::ConnectOptions;
        if !options.sqlx_logging {
            opt.disable_statement_logging();
        } else {
            opt.log_statements(options.sqlx_logging_level);
        }
        match options.pool_options().connect_with(opt).await {
            Ok(pool) => Ok(DatabaseConnection::SqlxMySqlPoolConnection(
                SqlxMySqlPoolConnection {
                    pool,
                    metric_callback: None,
                },
            )),
            Err(e) => Err(sqlx_error_to_conn_err(e)),
        }
    }
}

impl SqlxMySqlConnector {
    /// Instantiate a sqlx pool connection to a [DatabaseConnection]
    pub fn from_sqlx_mysql_pool(pool: MySqlPool) -> DatabaseConnection {
        DatabaseConnection::SqlxMySqlPoolConnection(SqlxMySqlPoolConnection {
            pool,
            metric_callback: None,
        })
    }
}

impl SqlxMySqlPoolConnection {
    /// Execute a [Statement] on a MySQL backend
    #[instrument(level = "trace")]
    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        let conn = &mut self.pool.acquire().await.map_err(conn_acquire_err)?;
        crate::metric::metric!(self.metric_callback, &stmt, {
            match query.execute(conn).await {
                Ok(res) => Ok(res.into()),
                Err(err) => Err(sqlx_error_to_exec_err(err)),
            }
        })
    }

    /// Execute an unprepared SQL statement on a MySQL backend
    #[instrument(level = "trace")]
    pub async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        debug_print!("{}", sql);

        let mut result = self.pool.acquire().await;
        if let Ok(conn) = &mut result {
            match conn.execute(sql).await {
                Ok(res) => Ok(res.into()),
                Err(err) => Err(sqlx_error_to_exec_err(err)),
            }
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout))
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))),
            }
        }
    }

    /// Get one result from a SQL query. Returns [Option::None] if no match was found
    #[instrument(level = "trace")]
    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        let mut result = self.pool.acquire().await;
        if let Ok(conn) = &mut result {
            crate::metric::metric!(self.metric_callback, &stmt, {
                match query.fetch_one(conn).await {
                    Ok(row) => Ok(Some(row.into())),
                    Err(err) => match err {
                        sqlx::Error::RowNotFound => Ok(None),
                        _ => Err(sqlx_error_to_query_err(err)),
                    },
                }
            })
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout))
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))),
            }
        }
    }

    /// Get the results of a query returning them as a Vec<[QueryResult]>
    #[instrument(level = "trace")]
    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        let mut result = self.pool.acquire().await;
        if let Ok(conn) = &mut result {
            crate::metric::metric!(self.metric_callback, &stmt, {
                match query.fetch_all(conn).await {
                    Ok(rows) => Ok(rows.into_iter().map(|r| r.into()).collect()),
                    Err(err) => Err(sqlx_error_to_query_err(err)),
                }
            })
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout))
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))),
            }
        }
    }

    /// Stream the results of executing a SQL query
    #[instrument(level = "trace")]
    pub async fn stream(&self, stmt: Statement) -> Result<QueryStream, DbErr> {
        debug_print!("{}", stmt);

        let result = self.pool.acquire().await;
        if let Ok(conn) = result {
            Ok(QueryStream::from((
                conn,
                stmt,
                self.metric_callback.clone(),
            )))
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout))
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))),
            }
        }
    }

    /// Bundle a set of SQL statements that execute together.
    #[instrument(level = "trace")]
    pub async fn begin(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        let result = self.pool.acquire().await;
        if let Ok(conn) = result {
            DatabaseTransaction::new_mysql(
                conn,
                self.metric_callback.clone(),
                isolation_level,
                access_mode,
            )
            .await
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout))
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))),
            }
        }
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
        E: std::error::Error + Send,
    {
        let result = self.pool.acquire().await;
        if let Ok(conn) = result {
            let transaction = DatabaseTransaction::new_mysql(
                conn,
                self.metric_callback.clone(),
                isolation_level,
                access_mode,
            )
            .await
            .map_err(|e| TransactionError::Connection(e))?;
            transaction.run(callback).await
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout).into())
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed).into())
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))
                .into()),
            }
        }
    }

    pub(crate) fn set_metric_callback<F>(&mut self, callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + Send + Sync + 'static,
    {
        self.metric_callback = Some(Arc::new(callback));
    }

    /// Checks if a connection to the database is still valid.
    pub async fn ping(&self) -> Result<(), DbErr> {
        let mut result = self.pool.acquire().await;
        if let Ok(conn) = &mut result {
            match conn.ping().await {
                Ok(_) => Ok(()),
                Err(err) => Err(sqlx_error_to_conn_err(err)),
            }
        } else {
            match result {
                Err(sqlx::Error::PoolTimedOut) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout))
                }
                Err(sqlx::Error::PoolClosed) => {
                    Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
                }
                _ => Err(DbErr::Conn(RuntimeErr::SqlxError(
                    result.expect_err("should be an error"),
                ))),
            }
        }
    }

    /// Explicitly close the MySQL connection
    pub async fn close(self) -> Result<(), DbErr> {
        self.pool.close().await;
        Ok(())
    }
}

impl From<MySqlRow> for QueryResult {
    fn from(row: MySqlRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxMySql(row),
        }
    }
}

impl From<MySqlQueryResult> for ExecResult {
    fn from(result: MySqlQueryResult) -> ExecResult {
        ExecResult {
            result: ExecResultHolder::SqlxMySql(result),
        }
    }
}

pub(crate) fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, MySql, SqlxValues> {
    let values = stmt
        .values
        .as_ref()
        .map_or(Values(Vec::new()), |values| values.clone());
    sqlx::query_with(&stmt.sql, SqlxValues(values))
}

pub(crate) async fn set_transaction_config(
    conn: &mut PoolConnection<MySql>,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
) -> Result<(), DbErr> {
    if let Some(isolation_level) = isolation_level {
        let stmt = Statement {
            sql: format!("SET TRANSACTION ISOLATION LEVEL {isolation_level}"),
            values: None,
            db_backend: DbBackend::MySql,
        };
        let query = sqlx_query(&stmt);
        conn.execute(query).await.map_err(sqlx_error_to_exec_err)?;
    }
    if let Some(access_mode) = access_mode {
        let stmt = Statement {
            sql: format!("SET TRANSACTION {access_mode}"),
            values: None,
            db_backend: DbBackend::MySql,
        };
        let query = sqlx_query(&stmt);
        conn.execute(query).await.map_err(sqlx_error_to_exec_err)?;
    }
    Ok(())
}
