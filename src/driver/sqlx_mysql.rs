use std::{future::Future, pin::Pin, sync::Arc};

use sqlx::{
    mysql::{MySqlArguments, MySqlConnectOptions, MySqlQueryResult, MySqlRow},
    MySql, MySqlPool,
};

sea_query::sea_query_driver_mysql!();
use sea_query_driver_mysql::bind_query;
use tracing::instrument;

use crate::{
    debug_print, error::*, executor::*, ConnectOptions, DatabaseConnection, DatabaseTransaction,
    QueryStream, Statement, TransactionError,
};

use super::sqlx_common::*;

/// Defines the [sqlx::mysql] connector
#[derive(Debug)]
pub struct SqlxMySqlConnector;

/// Defines a sqlx MySQL pool
#[derive(Clone)]
pub struct SqlxMySqlPoolConnection {
    pool: MySqlPool,
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
            .map_err(|e| DbErr::Conn(e.to_string()))?;
        if !options.sqlx_logging {
            use sqlx::ConnectOptions;
            opt.disable_statement_logging();
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
        if let Ok(conn) = &mut self.pool.acquire().await {
            crate::metric::metric!(self.metric_callback, &stmt, {
                match query.execute(conn).await {
                    Ok(res) => Ok(res.into()),
                    Err(err) => Err(sqlx_error_to_exec_err(err)),
                }
            })
        } else {
            Err(DbErr::Exec(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    /// Get one result from a SQL query. Returns [Option::None] if no match was found
    #[instrument(level = "trace")]
    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            crate::metric::metric!(self.metric_callback, &stmt, {
                match query.fetch_one(conn).await {
                    Ok(row) => Ok(Some(row.into())),
                    Err(err) => match err {
                        sqlx::Error::RowNotFound => Ok(None),
                        _ => Err(DbErr::Query(err.to_string())),
                    },
                }
            })
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    /// Get the results of a query returning them as a Vec<[QueryResult]>
    #[instrument(level = "trace")]
    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            crate::metric::metric!(self.metric_callback, &stmt, {
                match query.fetch_all(conn).await {
                    Ok(rows) => Ok(rows.into_iter().map(|r| r.into()).collect()),
                    Err(err) => Err(sqlx_error_to_query_err(err)),
                }
            })
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    /// Stream the results of executing a SQL query
    #[instrument(level = "trace")]
    pub async fn stream(&self, stmt: Statement) -> Result<QueryStream, DbErr> {
        debug_print!("{}", stmt);

        if let Ok(conn) = self.pool.acquire().await {
            Ok(QueryStream::from((
                conn,
                stmt,
                self.metric_callback.clone(),
            )))
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    /// Bundle a set of SQL statements that execute together.
    #[instrument(level = "trace")]
    pub async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        if let Ok(conn) = self.pool.acquire().await {
            DatabaseTransaction::new_mysql(conn, self.metric_callback.clone()).await
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    /// Create a MySQL transaction
    #[instrument(level = "trace", skip(callback))]
    pub async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(
                &'b DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>
            + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        if let Ok(conn) = self.pool.acquire().await {
            let transaction = DatabaseTransaction::new_mysql(conn, self.metric_callback.clone())
                .await
                .map_err(|e| TransactionError::Connection(e))?;
            transaction.run(callback).await
        } else {
            Err(TransactionError::Connection(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            )))
        }
    }

    pub(crate) fn set_metric_callback<F>(&mut self, callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + Send + Sync + 'static,
    {
        self.metric_callback = Some(Arc::new(callback));
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

pub(crate) fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, MySql, MySqlArguments> {
    let mut query = sqlx::query(&stmt.sql);
    if let Some(values) = &stmt.values {
        query = bind_query(query, values);
    }
    query
}
