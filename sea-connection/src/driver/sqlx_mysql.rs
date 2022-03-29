use std::{future::Future, pin::Pin, sync::Arc};

use sqlx::{
    mysql::{MySqlArguments, MySqlConnectOptions, MySqlQueryResult, MySqlRow},
    MySql, MySqlPool,
};

macro_rules! bind_params_sqlx_mysql {
    ( $query:expr, $params:expr ) => {{
        let mut query = $query;
        for value in $params.iter() {
            macro_rules! bind {
                ( $v: expr, $ty: ty ) => {
                    match $v {
                        Some(v) => query.bind((*v as $ty)),
                        None => query.bind(None::<$ty>),
                    }
                };
            }
            macro_rules! bind_box {
                ( $v: expr, $ty: ty ) => {
                    match $v {
                        Some(v) => query.bind(v.as_ref()),
                        None => query.bind(None::<$ty>),
                    }
                };
            }
            query = match value {
                Value::Bool(v) => bind!(v, bool),
                Value::TinyInt(v) => bind!(v, i8),
                Value::SmallInt(v) => bind!(v, i16),
                Value::Int(v) => bind!(v, i32),
                Value::BigInt(v) => bind!(v, i64),
                Value::TinyUnsigned(v) => bind!(v, u8),
                Value::SmallUnsigned(v) => bind!(v, u16),
                Value::Unsigned(v) => bind!(v, u32),
                Value::BigUnsigned(v) => bind!(v, u64),
                Value::Float(v) => bind!(v, f32),
                Value::Double(v) => bind!(v, f64),
                Value::String(v) => bind_box!(v, String),
                Value::Bytes(v) => bind_box!(v, Vec<u8>),
                _ => {
                    if value.is_json() {
                        query.bind(value.as_ref_json())
                    } else if value.is_chrono_date() {
                        query.bind(value.as_ref_chrono_date())
                    } else if value.is_chrono_time() {
                        query.bind(value.as_ref_chrono_time())
                    } else if value.is_chrono_date_time() {
                        query.bind(value.as_ref_chrono_date_time())
                    } else if value.is_chrono_date_time_utc() {
                        query.bind(value.as_ref_chrono_date_time_utc())
                    } else if value.is_chrono_date_time_local() {
                        query.bind(value.as_ref_chrono_date_time_local())
                    } else if value.is_chrono_date_time_with_time_zone() {
                        query.bind(value.chrono_as_naive_utc_in_string())
                    } else if value.is_time_date() {
                        query.bind(value.as_ref_time_date())
                    } else if value.is_time_time() {
                        query.bind(value.as_ref_time_time())
                    } else if value.is_time_date_time() {
                        query.bind(value.as_ref_time_date_time())
                    } else if value.is_time_date_time_with_time_zone() {
                        query.bind(value.time_as_naive_utc_in_string())
                    } else if value.is_decimal() {
                        query.bind(value.as_ref_decimal())
                    } else if value.is_big_decimal() {
                        query.bind(value.as_ref_big_decimal())
                    } else if value.is_uuid() {
                        query.bind(value.as_ref_uuid())
                    } else {
                        unimplemented!();
                    }
                }
            };
        }
        query
    }};
}

mod sea_query_driver_mysql {
    use sea_query::{Value, Values};
    use sqlx::{mysql::MySqlArguments, query::Query, query::QueryAs, MySql};

    type SqlxQuery<'a> = Query<'a, MySql, MySqlArguments>;
    type SqlxQueryAs<'a, T> = QueryAs<'a, MySql, T, MySqlArguments>;

    pub fn bind_query<'a>(query: SqlxQuery<'a>, params: &'a Values) -> SqlxQuery<'a> {
        bind_params_sqlx_mysql!(query, params.0)
    }

    pub fn bind_query_as<'a, T>(
        query: SqlxQueryAs<'a, T>,
        params: &'a Values,
    ) -> SqlxQueryAs<'a, T> {
        bind_params_sqlx_mysql!(query, params.0)
    }
}

// Why this isn't working??
// sea_query::sea_query_driver_mysql!();
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
