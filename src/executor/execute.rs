/// Result of a non-`SELECT` statement: an `INSERT`, `UPDATE`, `DELETE`, or
/// DDL execution. Carries the row count
/// ([`rows_affected`](Self::rows_affected)) and, where the backend supports
/// it, the last auto-generated primary key
/// ([`last_insert_id`](Self::last_insert_id)).
#[derive(Debug)]
pub struct ExecResult {
    pub(crate) result: ExecResultHolder,
}

/// Holds a result depending on the database backend chosen by the feature flag
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub(crate) enum ExecResultHolder {
    /// Holds the result of executing an operation on a MySQL database
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySql(sqlx::mysql::MySqlQueryResult),
    /// Holds the result of executing an operation on a PostgreSQL database
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgres(sqlx::postgres::PgQueryResult),
    /// Holds the result of executing an operation on a SQLite database
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlite(sqlx::sqlite::SqliteQueryResult),
    /// Holds the result of executing an operation on a SQLite database
    #[cfg(feature = "rusqlite")]
    Rusqlite(crate::driver::rusqlite::RusqliteExecResult),
    /// Holds the result of executing an operation on the Mock database
    #[cfg(feature = "mock")]
    Mock(crate::MockExecResult),
    /// Holds the result of executing an operation on the Proxy database
    #[cfg(feature = "proxy")]
    Proxy(crate::ProxyExecResult),
}

// ExecResult //

impl ExecResult {
    /// The auto-increment primary key value assigned by the database on the
    /// most recent `INSERT`.
    ///
    /// # Panics
    ///
    /// PostgreSQL does not expose `last_insert_id` directly — use
    /// `exec_with_returning` / `exec_with_returning_keys` instead.
    pub fn last_insert_id(&self) -> u64 {
        match &self.result {
            #[cfg(feature = "sqlx-mysql")]
            ExecResultHolder::SqlxMySql(result) => result.last_insert_id(),
            #[cfg(feature = "sqlx-postgres")]
            ExecResultHolder::SqlxPostgres(_) => {
                panic!("Should not retrieve last_insert_id this way")
            }
            #[cfg(feature = "sqlx-sqlite")]
            ExecResultHolder::SqlxSqlite(result) => {
                let last_insert_rowid = result.last_insert_rowid();
                if last_insert_rowid < 0 {
                    unreachable!("negative last_insert_rowid")
                } else {
                    last_insert_rowid as u64
                }
            }
            #[cfg(feature = "rusqlite")]
            ExecResultHolder::Rusqlite(result) => {
                let last_insert_rowid = result.last_insert_rowid;
                if last_insert_rowid < 0 {
                    unreachable!("negative last_insert_rowid")
                } else {
                    last_insert_rowid as u64
                }
            }
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => result.last_insert_id,
            #[cfg(feature = "proxy")]
            ExecResultHolder::Proxy(result) => result.last_insert_id,
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    /// Number of rows affected by the statement.
    pub fn rows_affected(&self) -> u64 {
        match &self.result {
            #[cfg(feature = "sqlx-mysql")]
            ExecResultHolder::SqlxMySql(result) => result.rows_affected(),
            #[cfg(feature = "sqlx-postgres")]
            ExecResultHolder::SqlxPostgres(result) => result.rows_affected(),
            #[cfg(feature = "sqlx-sqlite")]
            ExecResultHolder::SqlxSqlite(result) => result.rows_affected(),
            #[cfg(feature = "rusqlite")]
            ExecResultHolder::Rusqlite(result) => result.rows_affected,
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => result.rows_affected,
            #[cfg(feature = "proxy")]
            ExecResultHolder::Proxy(result) => result.rows_affected,
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}
