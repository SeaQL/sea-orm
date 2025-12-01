use futures_util::lock::Mutex;
use log::LevelFilter;
use sea_query::Values;
use std::{fmt::Write, future::Future, pin::Pin, sync::Arc};

use sqlx::{
    Connection, Executor, PgPool, Postgres,
    pool::PoolConnection,
    postgres::{PgConnectOptions, PgQueryResult, PgRow},
};

use sea_query_sqlx::SqlxValues;
use tracing::instrument;

use crate::{
    AccessMode, ConnectOptions, DatabaseConnection, DatabaseConnectionType, DatabaseTransaction,
    DbBackend, IsolationLevel, QueryStream, Statement, TransactionError, debug_print, error::*,
    executor::*,
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

impl From<PgPool> for SqlxPostgresPoolConnection {
    fn from(pool: PgPool) -> Self {
        SqlxPostgresPoolConnection {
            pool,
            metric_callback: None,
        }
    }
}

impl From<PgPool> for DatabaseConnection {
    fn from(pool: PgPool) -> Self {
        DatabaseConnectionType::SqlxPostgresPoolConnection(pool.into()).into()
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
        let mut sqlx_opts = options
            .url
            .parse::<PgConnectOptions>()
            .map_err(sqlx_error_to_conn_err)?;
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

        if let Some(f) = &options.pg_opts_fn {
            sqlx_opts = f(sqlx_opts);
        }

        let set_search_path_sql = options.schema_search_path.as_ref().map(|schema| {
            let mut string = "SET search_path = ".to_owned();
            if schema.starts_with('"') {
                write!(&mut string, "{schema}").expect("Infallible");
            } else {
                for (i, schema) in schema.split(',').enumerate() {
                    if i > 0 {
                        write!(&mut string, ",").expect("Infallible");
                    }
                    if schema.starts_with('"') {
                        write!(&mut string, "{schema}").expect("Infallible");
                    } else {
                        write!(&mut string, "\"{schema}\"").expect("Infallible");
                    }
                }
            }
            string
        });

        let lazy = options.connect_lazy;
        let after_connect = options.after_connect.clone();
        let mut pool_options = options.sqlx_pool_options();

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

        let pool = if lazy {
            pool_options.connect_lazy_with(sqlx_opts)
        } else {
            pool_options
                .connect_with(sqlx_opts)
                .await
                .map_err(sqlx_error_to_conn_err)?
        };

        let conn: DatabaseConnection =
            DatabaseConnectionType::SqlxPostgresPoolConnection(SqlxPostgresPoolConnection {
                pool,
                metric_callback: None,
            })
            .into();

        if let Some(cb) = after_connect {
            cb(conn.clone()).await?;
        }

        Ok(conn)
    }
}

impl SqlxPostgresConnector {
    /// Instantiate a sqlx pool connection to a [DatabaseConnection]
    pub fn from_sqlx_postgres_pool(pool: PgPool) -> DatabaseConnection {
        DatabaseConnectionType::SqlxPostgresPoolConnection(SqlxPostgresPoolConnection {
            pool,
            metric_callback: None,
        })
        .into()
    }
}

impl SqlxPostgresPoolConnection {
    /// Execute a [Statement] on a PostgreSQL backend
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

    /// Execute an unprepared SQL statement on a PostgreSQL backend
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
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        let conn = self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
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
        let conn = &mut self.pool.acquire().await.map_err(sqlx_conn_acquire_err)?;
        match conn.ping().await {
            Ok(_) => Ok(()),
            Err(err) => Err(sqlx_error_to_conn_err(err)),
        }
    }

    /// Explicitly close the Postgres connection.
    /// See [`Self::close_by_ref`] for usage with references.
    pub async fn close(self) -> Result<(), DbErr> {
        self.close_by_ref().await
    }

    /// Explicitly close the Postgres connection
    pub async fn close_by_ref(&self) -> Result<(), DbErr> {
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

impl
    From<(
        PoolConnection<sqlx::Postgres>,
        Statement,
        Option<crate::metric::Callback>,
    )> for crate::QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            PoolConnection<sqlx::Postgres>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        crate::QueryStream::build(
            stmt,
            crate::InnerConnection::Postgres(conn),
            metric_callback,
        )
    }
}

impl crate::DatabaseTransaction {
    pub(crate) async fn new_postgres(
        inner: PoolConnection<sqlx::Postgres>,
        metric_callback: Option<crate::metric::Callback>,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<crate::DatabaseTransaction, DbErr> {
        Self::begin(
            Arc::new(Mutex::new(crate::InnerConnection::Postgres(inner))),
            crate::DbBackend::Postgres,
            metric_callback,
            isolation_level,
            access_mode,
        )
        .await
    }
}

#[cfg(feature = "proxy")]
pub(crate) fn from_sqlx_postgres_row_to_proxy_row(row: &sqlx::postgres::PgRow) -> crate::ProxyRow {
    // https://docs.rs/sqlx-postgres/0.7.2/src/sqlx_postgres/type_info.rs.html
    // https://docs.rs/sqlx-postgres/0.7.2/sqlx_postgres/types/index.html
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
                        "BOOL" => {
                            Value::Bool(row.try_get(c.ordinal()).expect("Failed to get boolean"))
                        }
                        #[cfg(feature = "postgres-array")]
                        "BOOL[]" => Value::Array(
                            sea_query::ArrayType::Bool,
                            row.try_get::<Option<Vec<bool>>, _>(c.ordinal())
                                .expect("Failed to get boolean array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Bool(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "\"CHAR\"" => Value::TinyInt(
                            row.try_get(c.ordinal())
                                .expect("Failed to get small integer"),
                        ),
                        #[cfg(feature = "postgres-array")]
                        "\"CHAR\"[]" => Value::Array(
                            sea_query::ArrayType::TinyInt,
                            row.try_get::<Option<Vec<i8>>, _>(c.ordinal())
                                .expect("Failed to get small integer array")
                                .map(|vals: Vec<i8>| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::TinyInt(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "SMALLINT" | "SMALLSERIAL" | "INT2" => Value::SmallInt(
                            row.try_get(c.ordinal())
                                .expect("Failed to get small integer"),
                        ),
                        #[cfg(feature = "postgres-array")]
                        "SMALLINT[]" | "SMALLSERIAL[]" | "INT2[]" => Value::Array(
                            sea_query::ArrayType::SmallInt,
                            row.try_get::<Option<Vec<i16>>, _>(c.ordinal())
                                .expect("Failed to get small integer array")
                                .map(|vals: Vec<i16>| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::SmallInt(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "INT" | "SERIAL" | "INT4" => {
                            Value::Int(row.try_get(c.ordinal()).expect("Failed to get integer"))
                        }
                        #[cfg(feature = "postgres-array")]
                        "INT[]" | "SERIAL[]" | "INT4[]" => Value::Array(
                            sea_query::ArrayType::Int,
                            row.try_get::<Option<Vec<i32>>, _>(c.ordinal())
                                .expect("Failed to get integer array")
                                .map(|vals: Vec<i32>| {
                                    Box::new(
                                        vals.into_iter().map(|val| Value::Int(Some(val))).collect(),
                                    )
                                }),
                        ),

                        "BIGINT" | "BIGSERIAL" | "INT8" => Value::BigInt(
                            row.try_get(c.ordinal()).expect("Failed to get big integer"),
                        ),
                        #[cfg(feature = "postgres-array")]
                        "BIGINT[]" | "BIGSERIAL[]" | "INT8[]" => Value::Array(
                            sea_query::ArrayType::BigInt,
                            row.try_get::<Option<Vec<i64>>, _>(c.ordinal())
                                .expect("Failed to get big integer array")
                                .map(|vals: Vec<i64>| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::BigInt(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "FLOAT4" | "REAL" => {
                            Value::Float(row.try_get(c.ordinal()).expect("Failed to get float"))
                        }
                        #[cfg(feature = "postgres-array")]
                        "FLOAT4[]" | "REAL[]" => Value::Array(
                            sea_query::ArrayType::Float,
                            row.try_get::<Option<Vec<f32>>, _>(c.ordinal())
                                .expect("Failed to get float array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Float(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "FLOAT8" | "DOUBLE PRECISION" => {
                            Value::Double(row.try_get(c.ordinal()).expect("Failed to get double"))
                        }
                        #[cfg(feature = "postgres-array")]
                        "FLOAT8[]" | "DOUBLE PRECISION[]" => Value::Array(
                            sea_query::ArrayType::Double,
                            row.try_get::<Option<Vec<f64>>, _>(c.ordinal())
                                .expect("Failed to get double array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Double(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "VARCHAR" | "CHAR" | "TEXT" | "NAME" => Value::String(
                            row.try_get::<Option<String>, _>(c.ordinal())
                                .expect("Failed to get string")
                                .map(Box::new),
                        ),
                        #[cfg(feature = "postgres-array")]
                        "VARCHAR[]" | "CHAR[]" | "TEXT[]" | "NAME[]" => Value::Array(
                            sea_query::ArrayType::String,
                            row.try_get::<Option<Vec<String>>, _>(c.ordinal())
                                .expect("Failed to get string array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::String(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        "BYTEA" => Value::Bytes(
                            row.try_get::<Option<Vec<u8>>, _>(c.ordinal())
                                .expect("Failed to get bytes")
                                .map(Box::new),
                        ),
                        #[cfg(feature = "postgres-array")]
                        "BYTEA[]" => Value::Array(
                            sea_query::ArrayType::Bytes,
                            row.try_get::<Option<Vec<Vec<u8>>>, _>(c.ordinal())
                                .expect("Failed to get bytes array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Bytes(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-bigdecimal")]
                        "NUMERIC" => Value::BigDecimal(
                            row.try_get::<Option<bigdecimal::BigDecimal>, _>(c.ordinal())
                                .expect("Failed to get numeric")
                                .map(Box::new),
                        ),
                        #[cfg(all(
                            feature = "with-rust_decimal",
                            not(feature = "with-bigdecimal")
                        ))]
                        "NUMERIC" => Value::Decimal(
                            row.try_get(c.ordinal())
                                .expect("Failed to get numeric")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-bigdecimal", feature = "postgres-array"))]
                        "NUMERIC[]" => Value::Array(
                            sea_query::ArrayType::BigDecimal,
                            row.try_get::<Option<Vec<bigdecimal::BigDecimal>>, _>(c.ordinal())
                                .expect("Failed to get numeric array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::BigDecimal(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),
                        #[cfg(all(
                            feature = "with-rust_decimal",
                            not(feature = "with-bigdecimal"),
                            feature = "postgres-array"
                        ))]
                        "NUMERIC[]" => Value::Array(
                            sea_query::ArrayType::Decimal,
                            row.try_get::<Option<Vec<rust_decimal::Decimal>>, _>(c.ordinal())
                                .expect("Failed to get numeric array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Decimal(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        "OID" => {
                            Value::BigInt(row.try_get(c.ordinal()).expect("Failed to get oid"))
                        }
                        #[cfg(feature = "postgres-array")]
                        "OID[]" => Value::Array(
                            sea_query::ArrayType::BigInt,
                            row.try_get::<Option<Vec<i64>>, _>(c.ordinal())
                                .expect("Failed to get oid array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::BigInt(Some(val)))
                                            .collect(),
                                    )
                                }),
                        ),

                        "JSON" | "JSONB" => Value::Json(
                            row.try_get::<Option<serde_json::Value>, _>(c.ordinal())
                                .expect("Failed to get json")
                                .map(Box::new),
                        ),
                        #[cfg(any(feature = "json-array", feature = "postgres-array"))]
                        "JSON[]" | "JSONB[]" => Value::Array(
                            sea_query::ArrayType::Json,
                            row.try_get::<Option<Vec<serde_json::Value>>, _>(c.ordinal())
                                .expect("Failed to get json array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Json(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-ipnetwork")]
                        "INET" | "CIDR" => Value::IpNetwork(
                            row.try_get::<Option<ipnetwork::IpNetwork>, _>(c.ordinal())
                                .expect("Failed to get ip address")
                                .map(Box::new),
                        ),
                        #[cfg(feature = "with-ipnetwork")]
                        "INET[]" | "CIDR[]" => Value::Array(
                            sea_query::ArrayType::IpNetwork,
                            row.try_get::<Option<Vec<ipnetwork::IpNetwork>>, _>(c.ordinal())
                                .expect("Failed to get ip address array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::IpNetwork(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-mac_address")]
                        "MACADDR" | "MACADDR8" => Value::MacAddress(
                            row.try_get::<Option<mac_address::MacAddress>, _>(c.ordinal())
                                .expect("Failed to get mac address")
                                .map(Box::new),
                        ),
                        #[cfg(all(feature = "with-mac_address", feature = "postgres-array"))]
                        "MACADDR[]" | "MACADDR8[]" => Value::Array(
                            sea_query::ArrayType::MacAddress,
                            row.try_get::<Option<Vec<mac_address::MacAddress>>, _>(c.ordinal())
                                .expect("Failed to get mac address array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::MacAddress(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIMESTAMP" => Value::ChronoDateTime(
                            row.try_get::<Option<chrono::NaiveDateTime>, _>(c.ordinal())
                                .expect("Failed to get timestamp")
                                .map(Box::new),
                        ),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMESTAMP" => Value::TimeDateTime(
                            row.try_get::<Option<time::PrimitiveDateTime>, _>(c.ordinal())
                                .expect("Failed to get timestamp")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIMESTAMP[]" => Value::Array(
                            sea_query::ArrayType::ChronoDateTime,
                            row.try_get::<Option<Vec<chrono::NaiveDateTime>>, _>(c.ordinal())
                                .expect("Failed to get timestamp array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::ChronoDateTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIMESTAMP[]" => Value::Array(
                            sea_query::ArrayType::TimeDateTime,
                            row.try_get::<Option<Vec<time::PrimitiveDateTime>>, _>(c.ordinal())
                                .expect("Failed to get timestamp array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::TimeDateTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "DATE" => Value::ChronoDate(
                            row.try_get::<Option<chrono::NaiveDate>, _>(c.ordinal())
                                .expect("Failed to get date")
                                .map(Box::new),
                        ),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "DATE" => Value::TimeDate(
                            row.try_get::<Option<time::Date>, _>(c.ordinal())
                                .expect("Failed to get date")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "DATE[]" => Value::Array(
                            sea_query::ArrayType::ChronoDate,
                            row.try_get::<Option<Vec<chrono::NaiveDate>>, _>(c.ordinal())
                                .expect("Failed to get date array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::ChronoDate(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "DATE[]" => Value::Array(
                            sea_query::ArrayType::TimeDate,
                            row.try_get::<Option<Vec<time::Date>>, _>(c.ordinal())
                                .expect("Failed to get date array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::TimeDate(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIME" => Value::ChronoTime(
                            row.try_get::<Option<chrono::NaiveTime>, _>(c.ordinal())
                                .expect("Failed to get time")
                                .map(Box::new),
                        ),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIME" => Value::TimeTime(
                            row.try_get::<Option<time::Time>, _>(c.ordinal())
                                .expect("Failed to get time")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIME[]" => Value::Array(
                            sea_query::ArrayType::ChronoTime,
                            row.try_get::<Option<Vec<chrono::NaiveTime>>, _>(c.ordinal())
                                .expect("Failed to get time array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::ChronoTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIME[]" => Value::Array(
                            sea_query::ArrayType::TimeTime,
                            row.try_get::<Option<Vec<time::Time>>, _>(c.ordinal())
                                .expect("Failed to get time array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::TimeTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIMESTAMPTZ" => Value::ChronoDateTimeUtc(
                            row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(c.ordinal())
                                .expect("Failed to get timestamptz")
                                .map(Box::new),
                        ),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMESTAMPTZ" => Value::TimeDateTime(
                            row.try_get::<Option<time::PrimitiveDateTime>, _>(c.ordinal())
                                .expect("Failed to get timestamptz")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIMESTAMPTZ[]" => Value::Array(
                            sea_query::ArrayType::ChronoDateTimeUtc,
                            row.try_get::<Option<Vec<chrono::DateTime<chrono::Utc>>>, _>(
                                c.ordinal(),
                            )
                            .expect("Failed to get timestamptz array")
                            .map(|vals| {
                                Box::new(
                                    vals.into_iter()
                                        .map(|val| Value::ChronoDateTimeUtc(Some(Box::new(val))))
                                        .collect(),
                                )
                            }),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIMESTAMPTZ[]" => Value::Array(
                            sea_query::ArrayType::TimeDateTime,
                            row.try_get::<Option<Vec<time::PrimitiveDateTime>>, _>(c.ordinal())
                                .expect("Failed to get timestamptz array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::TimeDateTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-chrono")]
                        "TIMETZ" => Value::ChronoTime(
                            row.try_get::<Option<chrono::NaiveTime>, _>(c.ordinal())
                                .expect("Failed to get timetz")
                                .map(Box::new),
                        ),
                        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
                        "TIMETZ" => Value::TimeTime(
                            row.try_get(c.ordinal())
                                .expect("Failed to get timetz")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                        "TIMETZ[]" => Value::Array(
                            sea_query::ArrayType::ChronoTime,
                            row.try_get::<Option<Vec<chrono::NaiveTime>>, _>(c.ordinal())
                                .expect("Failed to get timetz array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::ChronoTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),
                        #[cfg(all(
                            feature = "with-time",
                            not(feature = "with-chrono"),
                            feature = "postgres-array"
                        ))]
                        "TIMETZ[]" => Value::Array(
                            sea_query::ArrayType::TimeTime,
                            row.try_get::<Option<Vec<time::Time>>, _>(c.ordinal())
                                .expect("Failed to get timetz array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::TimeTime(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        #[cfg(feature = "with-uuid")]
                        "UUID" => Value::Uuid(
                            row.try_get::<Option<uuid::Uuid>, _>(c.ordinal())
                                .expect("Failed to get uuid")
                                .map(Box::new),
                        ),

                        #[cfg(all(feature = "with-uuid", feature = "postgres-array"))]
                        "UUID[]" => Value::Array(
                            sea_query::ArrayType::Uuid,
                            row.try_get::<Option<Vec<uuid::Uuid>>, _>(c.ordinal())
                                .expect("Failed to get uuid array")
                                .map(|vals| {
                                    Box::new(
                                        vals.into_iter()
                                            .map(|val| Value::Uuid(Some(Box::new(val))))
                                            .collect(),
                                    )
                                }),
                        ),

                        _ => unreachable!("Unknown column type: {}", c.type_info().name()),
                    },
                )
            })
            .collect(),
    }
}
