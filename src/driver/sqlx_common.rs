use crate::DbErr;
use std::sync::Arc;

/// Converts an [sqlx::error] execution error to a [DbErr]
pub fn sqlx_error_to_exec_err(err: sqlx::Error) -> DbErr {
    DbErr::Exec(err.to_string(), Some(Arc::new(err)))
}

/// Converts an [sqlx::error] query error to a [DbErr]
pub fn sqlx_error_to_query_err(err: sqlx::Error) -> DbErr {
    DbErr::Query(err.to_string(), Some(Arc::new(err)))
}

/// Converts an [sqlx::error] connection error to a [DbErr]
pub fn sqlx_error_to_conn_err(err: sqlx::Error) -> DbErr {
    DbErr::Conn(err.to_string())
}
