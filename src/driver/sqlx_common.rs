use crate::DbErr;

/// Converts an [sqlx::error] execution error to a [DbErr]
pub fn sqlx_error_to_exec_err(err: sqlx::Error) -> DbErr {
    #[cfg(feature = "sqlx-error")]
    return sqlx_error_to_sqlx_db_err(err);

    DbErr::Exec(err.to_string())
}

/// Converts an [sqlx::error] query error to a [DbErr]
pub fn sqlx_error_to_query_err(err: sqlx::Error) -> DbErr {
    #[cfg(feature = "sqlx-error")]
    return sqlx_error_to_sqlx_db_err(err);

    DbErr::Query(err.to_string())
}

/// Converts an [sqlx::error] connection error to a [DbErr]
pub fn sqlx_error_to_conn_err(err: sqlx::Error) -> DbErr {
    #[cfg(feature = "sqlx-error")]
    return sqlx_error_to_sqlx_db_err(err);

    DbErr::Conn(err.to_string())
}

/// Converts an [sqlx::error] error to a [DbErr]
#[cfg(feature = "sqlx-error")]
pub fn sqlx_error_to_sqlx_db_err(err: sqlx::Error) -> DbErr {
    DbErr::Sqlx(err.into())
}

#[cfg(test)]
#[cfg(feature = "sqlx-error")]
mod tests {
    use crate::{sqlx_error_to_sqlx_db_err, DbErr, ErrFromSqlx};

    #[test]
    fn test_convert_with_sqlx_error_feature() {
        let expected = DbErr::Sqlx(ErrFromSqlx::from(sqlx::Error::RowNotFound));
        assert_eq!(
            expected,
            sqlx_error_to_sqlx_db_err(sqlx::Error::RowNotFound)
        )
    }
}
