#[derive(Debug)]
pub struct ExecResult {
    pub(crate) result: ExecResultHolder,
}

#[derive(Debug)]
pub(crate) enum ExecResultHolder {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySql(sqlx::mysql::MySqlQueryResult),
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgres(sqlx::postgres::PgQueryResult),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqlite(sqlx::sqlite::SqliteQueryResult),
    #[cfg(feature = "mock")]
    Mock(crate::MockExecResult),
}

// ExecResult //

impl ExecResult {
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
                    panic!("negative last_insert_rowid")
                } else {
                    last_insert_rowid as u64
                }
            }
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => result.last_insert_id,
        }
    }

    pub fn rows_affected(&self) -> u64 {
        match &self.result {
            #[cfg(feature = "sqlx-mysql")]
            ExecResultHolder::SqlxMySql(result) => result.rows_affected(),
            #[cfg(feature = "sqlx-postgres")]
            ExecResultHolder::SqlxPostgres(result) => result.rows_affected(),
            #[cfg(feature = "sqlx-sqlite")]
            ExecResultHolder::SqlxSqlite(result) => result.rows_affected(),
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => result.rows_affected,
        }
    }
}
