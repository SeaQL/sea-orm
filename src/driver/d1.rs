//! Cloudflare D1 database driver for Sea-ORM
//!
//! This module provides native D1 support for Sea-ORM, allowing you to use
//! Sea-ORM directly with Cloudflare Workers without needing to implement
//! the `ProxyDatabaseTrait`.
//!
//! # Architecture Overview
//!
//! Due to `wasm-bindgen` futures not being `Send`, the standard `ConnectionTrait`
//! and `TransactionTrait` cannot be implemented for D1. The driver provides:
//!
//! - [`D1Connection`]: Direct database access via raw SQL statements
//! - [`D1QueryExecutor`]: Entity/ActiveRecord-style queries with `Entity::find()`
//!
//! # Usage Patterns
//!
//! ## 1. Direct SQL Access
//!
//! ```ignore
//! use sea_orm::{DbBackend, D1Connection, Statement, Value};
//!
//! let stmt = Statement::from_sql_and_values(
//!     DbBackend::Sqlite,
//!     "SELECT * FROM users WHERE id = ?",
//!     vec![Value::from(1)],
//! );
//! let users = d1_conn.query_all(stmt).await?;
//! ```
//!
//! ## 2. Entity Queries (via D1QueryExecutor)
//!
//! ```ignore
//! use sea_orm::{D1QueryExecutor, EntityTrait};
//!
//! let cakes: Vec<cake::Model> = d1_conn.find_all(cake::Entity::find()).await?;
//! ```
//!
//! # Supported Types
//!
//! D1 supports the following Sea-ORM value types:
//! - Numeric: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64
//! - String: String, &str
//! - Binary: Vec<u8> (stored as hex string)
//! - JSON: serde_json::Value
//! - DateTime: chrono::DateTime types (stored as RFC3339 strings)
//! - Decimal: rust_decimal::Decimal, bigdecimal::BigDecimal
//! - UUID: uuid::Uuid
//! - Network: ipnetwork::IpNetwork
//!
//! # Limitations
//!
//! - **Transactions**: D1 has limited transaction support. Use [`D1Connection::transaction()`]
//!   directly, but be aware D1 doesn't guarantee ACID transactions.
//! - **Streaming**: D1 does not support streaming queries. Use `query_all()` to load all results.
//! - **Join queries**: [`D1QueryExecutor`] only supports simple `Select<E>` queries.
//!   For joins, use raw SQL with [`D1Connection`].
//! - **No `ConnectionTrait`**: The standard Sea-ORM connection interface isn't available.
//!   Use [`D1Connection`] methods directly or [`D1QueryExecutor`] for Entity operations.

//! # Entity Support with D1QueryExecutor
//!
//! Due to `wasm-bindgen` futures not being `Send`, the standard `ConnectionTrait`
//! cannot be implemented for D1. However, you can still use Entity queries via
//! the [`D1QueryExecutor`] trait:
//!
//! ```ignore
//! use sea_orm::{EntityTrait, D1QueryExecutor};
//!
//! async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
//!     let d1 = env.d1("DB")?;
//!     let db = sea_orm::Database::connect_d1(d1).await?;
//!     let d1_conn = db.as_d1_connection();
//!
//!     // Use Entity::find() with D1!
//!     let cakes: Vec<cake::Model> = d1_conn.find_all(cake::Entity::find()).await?;
//!
//!     // Or find one
//!     let cake: Option<cake::Model> = d1_conn.find_one(cake::Entity::find_by_id(1)).await?;
//!
//!     // With filters
//!     let filtered: Vec<cake::Model> = d1_conn
//!         .find_all(cake::Entity::find().filter(cake::Column::Name.contains("chocolate")))
//!         .await?;
//!
//!     Ok(Response::ok("Hello")?)
//! }
//! ```

use futures_util::lock::Mutex;
use sea_query::Values;
use std::{pin::Pin, sync::Arc};
use tracing::instrument;
use worker::wasm_bindgen::JsValue;

use crate::{
    AccessMode, DatabaseConnection, DatabaseConnectionType, DatabaseTransaction, DbErr, ExecResult,
    FromQueryResult, IsolationLevel, QueryResult, Statement, TransactionError, Value, debug_print,
    error::*, executor::*,
};

/// D1 Connector for Sea-ORM
///
/// This struct is used to create a connection to a D1 database.
#[derive(Debug)]
pub struct D1Connector;

/// A D1 database connection
///
/// This wraps a `worker::d1::D1Database` instance using `Arc` for cheap cloning,
/// since D1 connections are stateless and can be shared across threads.
#[derive(Clone)]
pub struct D1Connection {
    pub(crate) d1: Arc<worker::d1::D1Database>,
    pub(crate) metric_callback: Option<crate::metric::Callback>,
}

impl std::fmt::Debug for D1Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "D1Connection {{ d1: Arc<worker::d1::D1Database> }}")
    }
}

impl From<worker::d1::D1Database> for D1Connection {
    fn from(d1: worker::d1::D1Database) -> Self {
        D1Connection {
            d1: Arc::new(d1),
            metric_callback: None,
        }
    }
}

/// Result from executing a D1 query
#[derive(Debug, Clone)]
pub struct D1ExecResult {
    /// The last inserted row ID
    pub last_insert_id: u64,
    /// The number of rows affected
    pub rows_affected: u64,
}

/// A row returned from D1
///
/// This wraps the raw D1 row data which comes as `serde_json::Value`.
#[derive(Debug, Clone)]
pub struct D1Row {
    pub(crate) row: serde_json::Value,
}

impl D1Connector {
    /// Create a connection to a D1 database
    ///
    /// This takes a `worker::d1::D1Database` instance directly, which you can obtain
    /// from the Cloudflare Workers environment.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let d1 = env.d1("DB")?;
    /// let db = D1Connector::connect(d1).await?;
    /// ```
    #[instrument(level = "trace")]
    pub async fn connect(d1: worker::d1::D1Database) -> Result<DatabaseConnection, DbErr> {
        let conn = D1Connection::from(d1);
        Ok(DatabaseConnectionType::D1Connection(conn).into())
    }
}

impl D1Connection {
    /// Execute a prepared statement on D1
    #[instrument(level = "trace")]
    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let sql = stmt.sql.clone();
        let values = stmt
            .values
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Values(Vec::new()));

        crate::metric::metric!(self.metric_callback, &stmt, {
            match self.execute_inner(&sql, &values, false).await {
                Ok(result) => Ok(result.into()),
                Err(err) => Err(d1_error_to_exec_err(err)),
            }
        })
    }

    /// Execute an unprepared SQL statement on D1
    #[instrument(level = "trace")]
    pub async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        debug_print!("{}", sql);

        let values = Values(Vec::new());

        match self.execute_inner(sql, &values, false).await {
            Ok(result) => Ok(result.into()),
            Err(err) => Err(d1_error_to_exec_err(err)),
        }
    }

    /// Query a single row from D1
    #[instrument(level = "trace")]
    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let sql = stmt.sql.clone();
        let values = stmt
            .values
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Values(Vec::new()));

        crate::metric::metric!(self.metric_callback, &stmt, {
            match self.query_inner(&sql, &values).await {
                Ok(rows) => Ok(rows.into_iter().next().map(|r| r.into())),
                Err(err) => Err(d1_error_to_query_err(err)),
            }
        })
    }

    /// Query all rows from D1
    #[instrument(level = "trace")]
    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let sql = stmt.sql.clone();
        let values = stmt
            .values
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Values(Vec::new()));

        crate::metric::metric!(self.metric_callback, &stmt, {
            match self.query_inner(&sql, &values).await {
                Ok(rows) => Ok(rows.into_iter().map(|r| r.into()).collect()),
                Err(err) => Err(d1_error_to_query_err(err)),
            }
        })
    }

    /// Begin a transaction
    #[instrument(level = "trace")]
    pub async fn begin(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        if isolation_level.is_some() {
            tracing::warn!("Setting isolation level in a D1 transaction isn't supported");
        }
        if access_mode.is_some() {
            tracing::warn!("Setting access mode in a D1 transaction isn't supported");
        }

        // D1 doesn't support explicit transactions in the traditional sense.
        // We'll use a no-op transaction that just commits/rollbacks immediately.
        // This is a limitation of D1's current API.
        DatabaseTransaction::new_d1(self.d1.clone(), self.metric_callback.clone()).await
    }

    /// Execute a function inside a transaction
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
        let transaction =
            DatabaseTransaction::new_d1(self.d1.clone(), self.metric_callback.clone())
                .await
                .map_err(|e| TransactionError::Connection(e))?;
        transaction.run(callback).await
    }

    /// Check if the connection is still valid
    pub async fn ping(&self) -> Result<(), DbErr> {
        // D1 doesn't have a ping method, so we execute a simple query
        // to check if the connection is still valid
        match self.query_inner("SELECT 1", &Values(Vec::new())).await {
            Ok(_) => Ok(()),
            Err(err) => Err(d1_error_to_conn_err(err)),
        }
    }

    /// Close the connection
    pub async fn close_by_ref(&self) -> Result<(), DbErr> {
        // D1 doesn't need explicit closing - it's managed by the worker runtime
        Ok(())
    }

    /// Internal method to execute SQL and get execution result
    async fn execute_inner(
        &self,
        sql: &str,
        values: &Values,
        _unprepared: bool,
    ) -> Result<D1ExecResult, D1Error> {
        let js_values = values_to_js_values(values)?;

        let prepared = self
            .d1
            .prepare(sql)
            .bind(&js_values)
            .map_err(|e| D1Error::Prepare(e.into()))?;

        let result = prepared
            .run()
            .await
            .map_err(|e| D1Error::Execute(e.into()))?;
        let meta = result.meta().map_err(|e| D1Error::Meta(e.into()))?;

        let (last_insert_id, rows_affected) = match meta {
            Some(m) => (
                m.last_row_id.unwrap_or(0) as u64,
                m.rows_written.unwrap_or(0) as u64,
            ),
            None => (0, 0),
        };

        Ok(D1ExecResult {
            last_insert_id,
            rows_affected,
        })
    }

    /// Internal method to query and get rows
    async fn query_inner(&self, sql: &str, values: &Values) -> Result<Vec<D1Row>, D1Error> {
        let js_values = values_to_js_values(values)?;

        let prepared = self
            .d1
            .prepare(sql)
            .bind(&js_values)
            .map_err(|e| D1Error::Prepare(e.into()))?;

        let result = prepared.all().await.map_err(|e| D1Error::Query(e.into()))?;

        if let Some(error) = result.error() {
            return Err(D1Error::Response(error.to_string()));
        }

        let results: Vec<serde_json::Value> =
            result.results().map_err(|e| D1Error::Results(e.into()))?;

        let rows: Vec<D1Row> = results.into_iter().map(|row| D1Row { row }).collect();

        Ok(rows)
    }
}

/// Set the metric callback for this connection
impl D1Connection {
    pub(crate) fn set_metric_callback<F>(&mut self, callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + Send + Sync + 'static,
    {
        self.metric_callback = Some(Arc::new(callback));
    }
}

impl From<D1Row> for QueryResult {
    fn from(row: D1Row) -> Self {
        QueryResult {
            row: QueryResultRow::D1(row),
        }
    }
}

impl From<D1ExecResult> for ExecResult {
    fn from(result: D1ExecResult) -> Self {
        ExecResult {
            result: ExecResultHolder::D1(result),
        }
    }
}

/// Internal D1 error type
#[derive(Debug, thiserror::Error)]
enum D1Error {
    #[error("D1 prepare error: {0:?}")]
    Prepare(JsValue),
    #[error("D1 execute error: {0:?}")]
    Execute(JsValue),
    #[error("D1 query error: {0:?}")]
    Query(JsValue),
    #[error("D1 response error: {0}")]
    Response(String),
    #[error("D1 meta error: {0:?}")]
    Meta(JsValue),
    #[error("D1 results error: {0:?}")]
    Results(JsValue),
}

/// Convert D1 values to JS values for binding
fn values_to_js_values(values: &Values) -> Result<Vec<JsValue>, D1Error> {
    values.0.iter().map(value_to_js_value).collect()
}

/// Convert a Sea-ORM Value to a JS Value for D1
fn value_to_js_value(val: &Value) -> Result<JsValue, D1Error> {
    match val {
        Value::Bool(Some(v)) => Ok(JsValue::from(*v)),
        Value::Int(Some(v)) => Ok(JsValue::from(*v)),
        Value::BigInt(Some(v)) => Ok(JsValue::from(v.to_string())),
        Value::SmallInt(Some(v)) => Ok(JsValue::from(*v)),
        Value::TinyInt(Some(v)) => Ok(JsValue::from(*v)),
        Value::Unsigned(Some(v)) => Ok(JsValue::from(*v)),
        Value::BigUnsigned(Some(v)) => Ok(JsValue::from(v.to_string())),
        Value::SmallUnsigned(Some(v)) => Ok(JsValue::from(*v)),
        Value::TinyUnsigned(Some(v)) => Ok(JsValue::from(*v)),
        Value::Float(Some(v)) => Ok(JsValue::from_f64(*v as f64)),
        Value::Double(Some(v)) => Ok(JsValue::from_f64(*v)),
        Value::String(Some(v)) => Ok(JsValue::from(v.as_str())),
        Value::Char(Some(v)) => Ok(JsValue::from(v.to_string())),
        Value::Bytes(Some(v)) => {
            // Convert bytes to hex string for D1
            let hex: String = v.iter().map(|byte| format!("{:02x}", byte)).collect();
            Ok(JsValue::from(format!("X'{}'", hex)))
        }
        Value::Json(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDate(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-chrono")]
        Value::ChronoTime(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTime(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeUtc(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeLocal(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-chrono")]
        Value::ChronoDateTimeWithTimeZone(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-time")]
        Value::TimeDate(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-time")]
        Value::TimeTime(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-time")]
        Value::TimeDateTime(Some(v)) => Ok(JsValue::from(v.to_string())),
        #[cfg(feature = "with-time")]
        Value::TimeDateTimeWithTimeZone(Some(v)) => Ok(JsValue::from(v.to_string())),
        // Unsupported types - log warning and return NULL
        val => {
            tracing::warn!(
                "D1 does not support value type {:?} - converting to NULL. \
                Consider using a supported type (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String, Vec<u8>, serde_json::Value)",
                val
            );
            Ok(JsValue::NULL)
        }
    }
}

/// Convert D1 error to DbErr for execution
fn d1_error_to_exec_err(err: D1Error) -> DbErr {
    DbErr::Query(RuntimeErr::Internal(format!("D1 execute error: {}", err)))
}

/// Convert D1 error to DbErr for queries
fn d1_error_to_query_err(err: D1Error) -> DbErr {
    DbErr::Query(RuntimeErr::Internal(format!("D1 query error: {}", err)))
}

/// Convert D1 error to DbErr for connection
fn d1_error_to_conn_err(err: D1Error) -> DbErr {
    DbErr::Conn(RuntimeErr::Internal(format!(
        "D1 connection error: {}",
        err
    )))
}

/// Convert D1 JSON row to Sea-ORM values
pub(crate) fn d1_row_to_values(row: &D1Row) -> Vec<(String, Value)> {
    let mut values = Vec::new();

    if let Some(obj) = row.row.as_object() {
        for (key, value) in obj {
            let sea_value = d1_json_to_value(value);
            values.push((key.clone(), sea_value));
        }
    }

    values
}

/// Convert D1 JSON value to Sea-ORM Value
fn d1_json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Bool(None),
        serde_json::Value::Bool(v) => Value::Bool(Some(*v)),
        serde_json::Value::Number(v) => {
            if let Some(i) = v.as_i64() {
                Value::BigInt(Some(i))
            } else if let Some(u) = v.as_u64() {
                Value::BigUnsigned(Some(u))
            } else if let Some(f) = v.as_f64() {
                Value::Double(Some(f))
            } else {
                Value::Double(None)
            }
        }
        serde_json::Value::String(v) => Value::String(Some(v.clone())),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            Value::Json(Some(Box::new(json.clone())))
        }
    }
}

impl D1Row {
    /// Try to get a value from this D1 row by column name or index
    pub fn try_get_by<I: crate::ColIdx>(&self, idx: I) -> Result<Value, crate::TryGetError> {
        let values = d1_row_to_values(self);
        let col_name = idx.as_str().ok_or_else(|| {
            crate::TryGetError::Null(format!("D1 row doesn't support numeric index: {:?}", idx))
        })?;

        values
            .iter()
            .find(|(name, _)| name == col_name)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| {
                crate::TryGetError::Null(format!("Column '{}' not found in D1 row", col_name))
            })
    }
}

impl crate::DatabaseTransaction {
    pub(crate) async fn new_d1(
        d1: Arc<worker::d1::D1Database>,
        metric_callback: Option<crate::metric::Callback>,
    ) -> Result<crate::DatabaseTransaction, DbErr> {
        Self::begin(
            Arc::new(Mutex::new(crate::InnerConnection::D1(d1))),
            crate::DbBackend::Sqlite,
            metric_callback,
            None,
            None,
        )
        .await
    }
}

/// A trait for executing Entity queries on D1.
///
/// This trait enables `Entity::find()` operations with D1 by providing
/// methods that take `Select<E>` directly and execute them on D1.
///
/// Due to `wasm-bindgen` futures not being `Send`, the standard `ConnectionTrait`
/// cannot be implemented for D1. This trait provides an alternative way to use
/// Entity queries with D1.
///
/// # Example
///
/// ```ignore
/// use sea_orm::{EntityTrait, D1QueryExecutor};
///
/// let cakes: Vec<cake::Model> = d1_conn.find_all(cake::Entity::find()).await?;
/// let cake: Option<cake::Model> = d1_conn.find_one(cake::Entity::find_by_id(1)).await?;
/// ```
///
/// # Limitations
///
/// - **Transactions**: D1 has limited transaction support. Use [`D1Connection::transaction()`]
///   directly for transactional operations.
/// - **Streaming**: D1 does not support streaming queries. Use `find_all()` to load all results.
/// - **Join queries**: Only simple `Select<E>` queries are supported, not `SelectTwo` or `SelectTwoMany`.
/// - **No `ConnectionTrait`**: This trait provides Entity query support but doesn't implement
///   the full `ConnectionTrait` interface.
///
/// For operations not covered by this trait, use [`D1Connection`] directly with
/// [`Statement`](crate::Statement) and the [`execute`](D1Connection::execute),
/// [`query_one`](D1Connection::query_one), and [`query_all`](D1Connection::query_all) methods.
pub trait D1QueryExecutor {
    /// Execute a `Select<E>` and return all matching models.
    ///
    /// This allows you to use `Entity::find()` with D1:
    ///
    /// ```ignore
    /// let cakes: Vec<cake::Model> = d1_conn.find_all(cake::Entity::find()).await?;
    /// ```
    ///
    /// # Ordering and Filtering
    ///
    /// ```ignore
    /// use sea_orm::{EntityTrait, QueryOrder};
    ///
    /// let cakes: Vec<cake::Model> = d1_conn
    ///     .find_all(
    ///         cake::Entity::find()
    ///             .filter(cake::Column::Name.contains("chocolate"))
    ///             .order_by_asc(cake::Column::Name)
    ///     )
    ///     .await?;
    /// ```
    fn find_all<E>(
        &self,
        select: crate::Select<E>,
    ) -> impl std::future::Future<Output = Result<Vec<E::Model>, DbErr>>
    where
        E: crate::EntityTrait;

    /// Execute a `Select<E>` and return at most one model.
    ///
    /// This is useful for `Entity::find_by_id()` or queries with limits:
    ///
    /// ```ignore
    /// let cake: Option<cake::Model> = d1_conn.find_one(cake::Entity::find_by_id(1)).await?;
    /// ```
    fn find_one<E>(
        &self,
        select: crate::Select<E>,
    ) -> impl std::future::Future<Output = Result<Option<E::Model>, DbErr>>
    where
        E: crate::EntityTrait;

    /// Build a `Statement` from a `Select<E>` for manual execution.
    ///
    /// This allows you to get the SQL statement for debugging or custom execution:
    ///
    /// ```ignore
    /// let stmt = d1_conn.build_statement(cake::Entity::find().filter(
    ///     cake::Column::Name.contains("chocolate")
    /// ));
    /// ```
    fn build_statement<E>(&self, select: crate::Select<E>) -> Statement
    where
        E: crate::EntityTrait;
}

impl D1QueryExecutor for D1Connection {
    #[allow(clippy::manual_async_fn)]
    fn find_all<E>(
        &self,
        select: crate::Select<E>,
    ) -> impl std::future::Future<Output = Result<Vec<E::Model>, DbErr>>
    where
        E: crate::EntityTrait,
    {
        async move {
            let stmt = self.build_statement(select);
            let results = self.query_all(stmt).await?;
            results
                .into_iter()
                .map(|row| E::Model::from_query_result(&row, ""))
                .collect()
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn find_one<E>(
        &self,
        select: crate::Select<E>,
    ) -> impl std::future::Future<Output = Result<Option<E::Model>, DbErr>>
    where
        E: crate::EntityTrait,
    {
        async move {
            let stmt = self.build_statement(select);
            let result = self.query_one(stmt).await?;
            match result {
                Some(row) => E::Model::from_query_result(&row, "").map(Some),
                None => Ok(None),
            }
        }
    }

    fn build_statement<E>(&self, select: crate::Select<E>) -> Statement
    where
        E: crate::EntityTrait,
    {
        crate::DbBackend::Sqlite.build(&select.query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test conversion of D1 JSON null to Sea-ORM Value
    #[test]
    fn test_d1_null_conversion() {
        let json = serde_json::Value::Null;
        let value = d1_json_to_value(&json);
        assert_eq!(value, Value::Bool(None));
    }

    /// Test conversion of D1 JSON bool to Sea-ORM Value
    #[test]
    fn test_d1_bool_conversion() {
        let json = serde_json::Value::Bool(true);
        let value = d1_json_to_value(&json);
        assert_eq!(value, Value::Bool(Some(true)));

        let json = serde_json::Value::Bool(false);
        let value = d1_json_to_value(&json);
        assert_eq!(value, Value::Bool(Some(false)));
    }

    /// Test conversion of D1 JSON number (i64) to Sea-ORM Value
    #[test]
    fn test_d1_i64_conversion() {
        let json = serde_json::json!(42);
        let value = d1_json_to_value(&json);
        assert_eq!(value, Value::BigInt(Some(42)));
    }

    /// Test conversion of D1 JSON number (u64) to Sea-ORM Value
    #[test]
    fn test_d1_u64_conversion() {
        // D1 returns numbers as i64 when they fit, test the u64 path
        let json = serde_json::json!(9999999999999999999u64);
        let value = d1_json_to_value(&json);
        assert!(matches!(value, Value::BigUnsigned(_)));
    }

    /// Test conversion of D1 JSON number (f64) to Sea-ORM Value
    #[test]
    fn test_d1_f64_conversion() {
        let json = serde_json::json!(3.14159);
        let value = d1_json_to_value(&json);
        assert_eq!(value, Value::Double(Some(3.14159)));
    }

    /// Test conversion of D1 JSON string to Sea-ORM Value
    #[test]
    fn test_d1_string_conversion() {
        let json = serde_json::json!("hello world");
        let value = d1_json_to_value(&json);
        assert_eq!(value, Value::String(Some("hello world".to_string())));
    }

    /// Test conversion of D1 JSON array to Sea-ORM Value (as JSON)
    #[test]
    fn test_d1_array_conversion() {
        let json = serde_json::json!([1, 2, 3]);
        let value = d1_json_to_value(&json);
        assert!(matches!(value, Value::Json(Some(_))));
    }

    /// Test conversion of D1 JSON object to Sea-ORM Value (as JSON)
    #[test]
    fn test_d1_object_conversion() {
        let json = serde_json::json!({"key": "value"});
        let value = d1_json_to_value(&json);
        assert!(matches!(value, Value::Json(Some(_))));
    }

    /// Test d1_row_to_values function
    #[test]
    fn test_d1_row_to_values() {
        let row = D1Row {
            row: serde_json::json!({
                "id": 1,
                "name": "Chocolate Cake",
                "price": 9.99,
                "available": true
            }),
        };

        let values = d1_row_to_values(&row);
        assert_eq!(values.len(), 4);

        let id_value = values.iter().find(|(k, _)| k == "id").unwrap().1.clone();
        assert_eq!(id_value, Value::BigInt(Some(1)));

        let name_value = values.iter().find(|(k, _)| k == "name").unwrap().1.clone();
        assert_eq!(
            name_value,
            Value::String(Some("Chocolate Cake".to_string()))
        );
    }

    /// Test D1Row try_get_by with valid column
    #[test]
    fn test_d1_row_try_get_by_valid() {
        let row = D1Row {
            row: serde_json::json!({
                "id": 42,
                "name": "Test"
            }),
        };

        let id_value = row.try_get_by("id").unwrap();
        assert_eq!(id_value, Value::BigInt(Some(42)));

        let name_value = row.try_get_by("name").unwrap();
        assert_eq!(name_value, Value::String(Some("Test".to_string())));
    }

    /// Test D1Row try_get_by with missing column
    #[test]
    fn test_d1_row_try_get_by_missing() {
        let row = D1Row {
            row: serde_json::json!({
                "id": 42
            }),
        };

        // Missing column should return an error, not panic
        let result = row.try_get_by("nonexistent");
        assert!(result.is_err());
    }

    /// Test D1ExecResult creation
    #[test]
    fn test_d1_exec_result() {
        let result = D1ExecResult {
            last_insert_id: 123,
            rows_affected: 5,
        };

        assert_eq!(result.last_insert_id, 123);
        assert_eq!(result.rows_affected, 5);
    }
}
