use crate::{DbErr, RuntimeErr};

/// Converts an [sqlx::error] connection error to a [DbErr]
pub fn sqlx_error_to_conn_err(err: sqlx::Error) -> DbErr {
    DbErr::Conn(RuntimeErr::SqlxError(err))
}
