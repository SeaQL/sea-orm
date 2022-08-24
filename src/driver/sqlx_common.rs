use crate::DbErr;

/// Converts an [sqlx::error] execution error to a [DbErr]
pub fn sqlx_error_to_exec_err(err: sqlx::Error) -> DbErr {
    DbErr::ExecSqlX(err)
}

/// Converts an [sqlx::error] query error to a [DbErr]
pub fn sqlx_error_to_query_err(err: sqlx::Error) -> DbErr {
    DbErr::QuerySqlX(err)
}

/// Converts an [sqlx::error] connection error to a [DbErr]
pub fn sqlx_error_to_conn_err(err: sqlx::Error) -> DbErr {
    DbErr::ConnSqlX(err)
}
