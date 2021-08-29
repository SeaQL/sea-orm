use crate::TryGetable;
use std::str::FromStr;

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
    pub fn last_insert_id<T>(&self) -> T
    where
        T: TryGetable + Default + FromStr,
    {
        match &self.result {
            #[cfg(feature = "sqlx-mysql")]
            ExecResultHolder::SqlxMySql(result) => result
                .last_insert_id()
                .to_string()
                .parse()
                .unwrap_or_default(),
            #[cfg(feature = "sqlx-postgres")]
            ExecResultHolder::SqlxPostgres(result) => {
                res.try_get("", "last_insert_id").unwrap_or_default()
            }
            #[cfg(feature = "sqlx-sqlite")]
            ExecResultHolder::SqlxSqlite(result) => {
                let last_insert_rowid = result.last_insert_rowid();
                if last_insert_rowid < 0 {
                    panic!("negative last_insert_rowid")
                } else {
                    last_insert_rowid.to_string().parse().unwrap_or_default()
                }
            }
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => result
                .last_insert_id
                .to_string()
                .parse()
                .unwrap_or_default(),
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
