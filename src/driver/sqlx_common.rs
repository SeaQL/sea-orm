use crate::{DbErr, RuntimeErr};

/// Converts an [sqlx::error] execution error to a [DbErr]
pub fn sqlx_error_to_exec_err(err: sqlx::Error) -> DbErr {
    DbErr::Exec(RuntimeErr::SqlxError(err))
}

/// Converts an [sqlx::error] query error to a [DbErr]
pub fn sqlx_error_to_query_err(err: sqlx::Error) -> DbErr {
    DbErr::Query(RuntimeErr::SqlxError(err))
}

/// Converts an [sqlx::error] connection error to a [DbErr]
pub fn sqlx_error_to_conn_err(err: sqlx::Error) -> DbErr {
    DbErr::Conn(RuntimeErr::SqlxError(err))
}
