use sea_query::Values;
use std::{future::Future, pin::Pin, sync::Arc};

use sqlx::{
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgQueryResult, PgRow},
    Connection, Executor, PgPool, Postgres,
};

use sea_query_binder::SqlxValues;
use tracing::instrument;

use crate::{
    debug_print, error::*, executor::*, AccessMode, ConnectOptions, DatabaseConnection,
    DatabaseTransaction, DbBackend, IsolationLevel, QueryStream, Statement, TransactionError,
};

use super::sqlx_common::*;

/// Defines the [sqlx::postgres] connector
#[derive(Debug)]
pub struct SqlxPostgresConnector;

/// Defines a sqlx PostgreSQL pool
#[derive(Clone)]
pub struct SqlxPostgresPoolConnection {
    pub(crate) pool: PgPool,
    metric_callback: Option<crate::metric::Callback>,
}

impl std::fmt::Debug for SqlxPostgresPoolConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SqlxPostgresPoolConnection {{ pool: {:?} }}", self.pool)
    }
}

impl SqlxPostgresConnector {
    /// Check if the URI provided corresponds to `postgres://` for a PostgreSQL database
    pub fn accepts(string: &str) -> bool {
        string.starts_with("postgres://") && string.parse::<PgConnectOptions>().is_ok()
    }

    /// Add configuration options for the PostgreSQL database
    #[instrument(level = "trace")]
    pub async fn connect(options: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
        let mut opt = options
            .url
            .parse::<PgConnectOptions>()
            .map_err(sqlx_error_to_conn_err)?;
        use sqlx::ConnectOptions;
        if !options.sqlx_logging {
            opt.disable_statement_logging();
        } else {
            opt.log_statements(options.sqlx_logging_level);
        }
        let set_search_path_sql = options
            .schema_search_path
            .as_ref()
            .map(|schema| format!("SET search_path = '{schema}'"));
        let mut pool_options = options.pool_options();
        if let Some(sql) = set_search_path_sql {
            pool_options = pool_options.after_connect(move |conn, _| {
                let sql = sql.clone();
                Box::pin(async move {
                    sqlx::Executor::execute(conn, sql.as_str())
                        .await
                        .map(|_| ())
                })
            });
        }
        match pool_options.connect_with(opt).await {
            Ok(pool) => Ok(DatabaseConnection::SqlxPostgresPoolConnection(
                SqlxPostgresPoolConnection {
                    pool,
                    metric_callback: None,
                },
            )),
            Err(e) => Err(sqlx_error_to_conn_err(e)),
        }
    }
}

impl SqlxPostgresConnector {
    /// Instantiate a sqlx pool connection to a [DatabaseConnection]
    pub fn from_sqlx_postgres_pool(pool: PgPool) -> DatabaseConnection {
        DatabaseConnection::SqlxPostgresPoolConnection(SqlxPostgresPoolConnection {
            pool,
            metric_callback: None,
        })
    }
}

impl SqlxPostgresPoolConnection {
    /// Execute a [Statement] on a PostgreSQL backend
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

    /// Execute an unprepared SQL statement on a PostgreSQL backend
    #[instrument(level = "trace")]
    pub async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        debug_print!("{}", sql);

        let conn = &mut self.pool.acquire().await.map_err(conn_acquire_err)?;
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
        let conn = &mut self.pool.acquire().await.map_err(conn_acquire_err)?;
        crate::metric::metric!(self.metric_callback, &stmt, {
            match query.fetch_one(conn).await {
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
        let conn = &mut self.pool.acquire().await.map_err(conn_acquire_err)?;
        crate::metric::metric!(self.metric_callback, &stmt, {
            match query.fetch_all(conn).await {
                Ok(rows) => Ok(rows.into_iter().map(|r| r.into()).collect()),
                Err(err) => Err(sqlx_error_to_query_err(err)),
            }
        })
    }

    /// Stream the results of executing a SQL query
    #[instrument(level = "trace")]
    pub async fn stream(&self, stmt: Statement) -> Result<QueryStream, DbErr> {
        debug_print!("{}", stmt);
        let conn = self.pool.acquire().await.map_err(conn_acquire_err)?;
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
        let conn = self.pool.acquire().await.map_err(conn_acquire_err)?;
        DatabaseTransaction::new_postgres(
            conn,
            self.metric_callback.clone(),
            isolation_level,
            access_mode,
        )
        .await
    }

    /// Create a PostgreSQL transaction
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
        let conn = self.pool.acquire().await.map_err(conn_acquire_err)?;
        let transaction = DatabaseTransaction::new_postgres(
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
        let conn = &mut self.pool.acquire().await.map_err(conn_acquire_err)?;
        match conn.ping().await {
            Ok(_) => Ok(()),
            Err(err) => Err(sqlx_error_to_conn_err(err)),
        }
    }

    /// Explicitly close the Postgres connection
    pub async fn close(self) -> Result<(), DbErr> {
        self.pool.close().await;
        Ok(())
    }
}

impl From<PgRow> for QueryResult {
    fn from(row: PgRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxPostgres(row),
        }
    }
}

impl From<PgQueryResult> for ExecResult {
    fn from(result: PgQueryResult) -> ExecResult {
        ExecResult {
            result: ExecResultHolder::SqlxPostgres(result),
        }
    }
}

pub(crate) fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, Postgres, SqlxValues> {
    let values = stmt
        .values
        .as_ref()
        .map_or(Values(Vec::new()), |values| values.clone());
    sqlx::query_with(&stmt.sql, SqlxValues(values))
}

pub(crate) async fn set_transaction_config(
    conn: &mut PoolConnection<Postgres>,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
) -> Result<(), DbErr> {
    if let Some(isolation_level) = isolation_level {
        let stmt = Statement {
            sql: format!("SET TRANSACTION ISOLATION LEVEL {isolation_level}"),
            values: None,
            db_backend: DbBackend::Postgres,
        };
        let query = sqlx_query(&stmt);
        conn.execute(query).await.map_err(sqlx_error_to_exec_err)?;
    }
    if let Some(access_mode) = access_mode {
        let stmt = Statement {
            sql: format!("SET TRANSACTION {access_mode}"),
            values: None,
            db_backend: DbBackend::Postgres,
        };
        let query = sqlx_query(&stmt);
        conn.execute(query).await.map_err(sqlx_error_to_exec_err)?;
    }
    Ok(())
}
