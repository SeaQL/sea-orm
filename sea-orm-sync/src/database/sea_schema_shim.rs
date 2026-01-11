use crate::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, QueryResult, QueryResultRow,
    RuntimeErr, Statement,
};
use sea_query::SelectStatement;
use sea_schema::sqlx_types::SqlxRow;
use sqlx::Error as SqlxError;
use std::sync::Arc;

impl sea_schema::Connection for DatabaseConnection {
    fn query_all(&self, select: SelectStatement) -> Result<Vec<SqlxRow>, SqlxError> {
        map_result(ConnectionTrait::query_all(self, &select))
    }

    fn query_all_raw(&self, sql: String) -> Result<Vec<SqlxRow>, SqlxError> {
        map_result(ConnectionTrait::query_all_raw(
            self,
            Statement::from_string(self.get_database_backend(), sql),
        ))
    }
}

impl sea_schema::Connection for DatabaseTransaction {
    fn query_all(&self, select: SelectStatement) -> Result<Vec<SqlxRow>, SqlxError> {
        map_result(ConnectionTrait::query_all(self, &select))
    }

    fn query_all_raw(&self, sql: String) -> Result<Vec<SqlxRow>, SqlxError> {
        map_result(ConnectionTrait::query_all_raw(
            self,
            Statement::from_string(self.get_database_backend(), sql),
        ))
    }
}

impl sea_schema::Connection for crate::DatabaseExecutor<'_> {
    fn query_all(&self, select: SelectStatement) -> Result<Vec<SqlxRow>, SqlxError> {
        match self {
            crate::DatabaseExecutor::Connection(conn) => {
                <DatabaseConnection as sea_schema::Connection>::query_all(conn, select)
            }
            crate::DatabaseExecutor::Transaction(txn) => {
                <DatabaseTransaction as sea_schema::Connection>::query_all(txn, select)
            }
        }
    }

    fn query_all_raw(&self, sql: String) -> Result<Vec<SqlxRow>, SqlxError> {
        match self {
            crate::DatabaseExecutor::Connection(conn) => {
                <DatabaseConnection as sea_schema::Connection>::query_all_raw(conn, sql)
            }
            crate::DatabaseExecutor::Transaction(txn) => {
                <DatabaseTransaction as sea_schema::Connection>::query_all_raw(txn, sql)
            }
        }
    }
}

fn map_result(result: Result<Vec<QueryResult>, DbErr>) -> Result<Vec<SqlxRow>, SqlxError> {
    match result {
        Ok(rows) => Ok(rows
            .into_iter()
            .filter_map(|r| match r.row {
                #[cfg(feature = "sqlx-mysql")]
                QueryResultRow::SqlxMySql(r) => Some(SqlxRow::MySql(r)),
                #[cfg(feature = "sqlx-postgres")]
                QueryResultRow::SqlxPostgres(r) => Some(SqlxRow::Postgres(r)),
                #[cfg(feature = "sqlx-sqlite")]
                QueryResultRow::SqlxSqlite(r) => Some(SqlxRow::Sqlite(r)),
                #[allow(unreachable_patterns)]
                _ => None,
            })
            .collect()),
        Err(err) => Err(match err {
            DbErr::Conn(RuntimeErr::SqlxError(err)) => {
                Arc::into_inner(err).expect("Should only have one owner")
            }
            DbErr::Exec(RuntimeErr::SqlxError(err)) => {
                Arc::into_inner(err).expect("Should only have one owner")
            }
            DbErr::Query(RuntimeErr::SqlxError(err)) => {
                Arc::into_inner(err).expect("Should only have one owner")
            }
            _ => SqlxError::AnyDriverError(Box::new(err)),
        }),
    }
}
