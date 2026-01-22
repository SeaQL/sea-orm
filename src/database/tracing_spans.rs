//! Tracing support for database operations.
//!
//! This module provides utilities for instrumenting database operations
//! with tracing spans. Enable the `tracing-spans` feature to automatically
//! generate spans for all database operations.
//!
//! # Example
//!
//! ```toml
//! [dependencies]
//! sea-orm = { version = "2.0", features = ["tracing-spans"] }
//! ```
//!
//! ```ignore
//! use sea_orm::Database;
//!
//! // Set up a tracing subscriber
//! tracing_subscriber::fmt()
//!     .with_max_level(tracing::Level::INFO)
//!     .init();
//!
//! // All database operations will now generate tracing spans
//! let db = Database::connect("sqlite::memory:").await?;
//! let cakes = Cake::find().all(&db).await?;  // Generates a span
//! ```

#[cfg(feature = "tracing-spans")]
mod inner {
    use crate::DbBackend;

    /// Database operation type, following OpenTelemetry conventions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum DbOperation {
        /// SELECT query
        Select,
        /// INSERT statement
        Insert,
        /// UPDATE statement
        Update,
        /// DELETE statement
        Delete,
        /// Other/unknown SQL execution
        Execute,
    }

    impl DbOperation {
        /// Parse the operation type from an SQL query string.
        ///
        /// This function is allocation-free and uses case-insensitive comparison.
        pub fn from_sql(sql: &str) -> Self {
            let first_word = sql.trim_start().split_whitespace().next().unwrap_or("");
            if first_word.eq_ignore_ascii_case("SELECT") {
                DbOperation::Select
            } else if first_word.eq_ignore_ascii_case("INSERT") {
                DbOperation::Insert
            } else if first_word.eq_ignore_ascii_case("UPDATE") {
                DbOperation::Update
            } else if first_word.eq_ignore_ascii_case("DELETE") {
                DbOperation::Delete
            } else {
                DbOperation::Execute
            }
        }
    }

    impl std::fmt::Display for DbOperation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                DbOperation::Select => write!(f, "SELECT"),
                DbOperation::Insert => write!(f, "INSERT"),
                DbOperation::Update => write!(f, "UPDATE"),
                DbOperation::Delete => write!(f, "DELETE"),
                DbOperation::Execute => write!(f, "EXECUTE"),
            }
        }
    }

    /// Get the OpenTelemetry system name from DbBackend.
    pub(crate) fn db_system_name(backend: DbBackend) -> &'static str {
        match backend {
            DbBackend::Postgres => "postgresql",
            DbBackend::MySql => "mysql",
            DbBackend::Sqlite => "sqlite",
        }
    }

    /// Record query result on a span (success/failure status and error message).
    pub(crate) fn record_query_result<T, E: std::fmt::Display>(
        span: &tracing::Span,
        result: &Result<T, E>,
    ) {
        match result {
            Ok(_) => {
                span.record("otel.status_code", "OK");
            }
            Err(e) => {
                span.record("otel.status_code", "ERROR");
                span.record("exception.message", tracing::field::display(e));
            }
        }
    }
}

#[cfg(feature = "tracing-spans")]
pub(crate) use inner::*;

/// Create a tracing span for database operations.
///
/// Arguments:
/// - `$name`: Span name (e.g., "sea_orm.execute", "sea_orm.query_one")
/// - `$backend`: DbBackend value
/// - `$sql`: SQL statement string (used for operation parsing)
///
/// Note: `db.statement` is set to Empty. Call `span.record("db.statement", sql)`
/// separately if the query is parameterized (safe to log).
#[cfg(feature = "tracing-spans")]
macro_rules! db_span {
    ($name:expr, $backend:expr, $sql:expr) => {{
        let sql: &str = $sql;
        let op = $crate::database::tracing_spans::DbOperation::from_sql(sql);
        ::tracing::info_span!(
            $name,
            db.system = $crate::database::tracing_spans::db_system_name($backend),
            db.operation = %op,
            db.statement = ::tracing::field::Empty,
            otel.status_code = ::tracing::field::Empty,
            exception.message = ::tracing::field::Empty,
        )
    }};
}

#[cfg(feature = "tracing-spans")]
pub(crate) use db_span;

/// Execute a future and wrap it in a tracing span when `tracing-spans` is enabled.
///
/// When the feature is disabled, this macro simply awaits the future with zero overhead.
///
/// # Arguments
/// - `$name`: span name (e.g., "sea_orm.execute")
/// - `$backend`: DbBackend
/// - `$sql`: &str used for db.operation parsing
/// - `record_stmt`: whether to record `db.statement`
/// - `$fut`: the future to execute
macro_rules! with_db_span {
    ($name:expr, $backend:expr, $sql:expr, record_stmt = $record_stmt:expr, $fut:expr) => {{
        #[cfg(all(feature = "tracing-spans", not(feature = "sync")))]
        {
            let span = $crate::database::tracing_spans::db_span!($name, $backend, $sql);
            if $record_stmt {
                span.record("db.statement", $sql);
            }
            let result = ::tracing::Instrument::instrument($fut, span.clone()).await;
            $crate::database::tracing_spans::record_query_result(&span, &result);
            result
        }
        #[cfg(all(feature = "tracing-spans", feature = "sync"))]
        {
            let span = $crate::database::tracing_spans::db_span!($name, $backend, $sql);
            if $record_stmt {
                span.record("db.statement", $sql);
            }
            span.in_scope($fut)
        }
        #[cfg(not(feature = "tracing-spans"))]
        {
            $fut.await
        }
    }};
}

pub(crate) use with_db_span;

#[cfg(feature = "tracing-spans")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_operation_from_sql() {
        assert_eq!(
            DbOperation::from_sql("SELECT * FROM users"),
            DbOperation::Select
        );
        assert_eq!(
            DbOperation::from_sql("  SELECT * FROM users"),
            DbOperation::Select
        );
        assert_eq!(
            DbOperation::from_sql("select * from users"),
            DbOperation::Select
        );
        assert_eq!(
            DbOperation::from_sql("INSERT INTO users"),
            DbOperation::Insert
        );
        assert_eq!(
            DbOperation::from_sql("UPDATE users SET"),
            DbOperation::Update
        );
        assert_eq!(
            DbOperation::from_sql("DELETE FROM users"),
            DbOperation::Delete
        );
        assert_eq!(
            DbOperation::from_sql("CREATE TABLE users"),
            DbOperation::Execute
        );
        assert_eq!(
            DbOperation::from_sql("DROP TABLE users"),
            DbOperation::Execute
        );
    }

    #[test]
    fn test_db_system_name() {
        assert_eq!(db_system_name(DbBackend::Postgres), "postgresql");
        assert_eq!(db_system_name(DbBackend::MySql), "mysql");
        assert_eq!(db_system_name(DbBackend::Sqlite), "sqlite");
    }

    #[test]
    fn test_db_operation_display() {
        assert_eq!(DbOperation::Select.to_string(), "SELECT");
        assert_eq!(DbOperation::Insert.to_string(), "INSERT");
        assert_eq!(DbOperation::Update.to_string(), "UPDATE");
        assert_eq!(DbOperation::Delete.to_string(), "DELETE");
        assert_eq!(DbOperation::Execute.to_string(), "EXECUTE");
    }
}
