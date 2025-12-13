use crate::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, QueryResult, QueryResultRow,
    RuntimeErr, Statement, driver::rusqlite::RusqliteRow as OurRusqliteRow,
};
use sea_query::SelectStatement;
use sea_schema::rusqlite_types::{RusqliteError, RusqliteRow};
use std::sync::Arc;

impl sea_schema::Connection for DatabaseConnection {
    fn query_all(&self, select: SelectStatement) -> Result<Vec<RusqliteRow>, RusqliteError> {
        map_result(ConnectionTrait::query_all(self, &select))
    }

    fn query_all_raw(&self, sql: String) -> Result<Vec<RusqliteRow>, RusqliteError> {
        map_result(ConnectionTrait::query_all_raw(
            self,
            Statement::from_string(self.get_database_backend(), sql),
        ))
    }
}

impl sea_schema::Connection for DatabaseTransaction {
    fn query_all(&self, select: SelectStatement) -> Result<Vec<RusqliteRow>, RusqliteError> {
        map_result(ConnectionTrait::query_all(self, &select))
    }

    fn query_all_raw(&self, sql: String) -> Result<Vec<RusqliteRow>, RusqliteError> {
        map_result(ConnectionTrait::query_all_raw(
            self,
            Statement::from_string(self.get_database_backend(), sql),
        ))
    }
}

fn map_result(result: Result<Vec<QueryResult>, DbErr>) -> Result<Vec<RusqliteRow>, RusqliteError> {
    match result {
        Ok(rows) => Ok(rows
            .into_iter()
            .filter_map(|r| match r.row {
                #[cfg(feature = "rusqlite")]
                QueryResultRow::Rusqlite(OurRusqliteRow { values, .. }) => {
                    Some(RusqliteRow { values })
                }
                #[allow(unreachable_patterns)]
                _ => None,
            })
            .collect()),
        Err(err) => Err(match err {
            DbErr::Conn(RuntimeErr::Rusqlite(err)) => {
                Arc::into_inner(err).expect("Should only have one owner")
            }
            DbErr::Exec(RuntimeErr::Rusqlite(err)) => {
                Arc::into_inner(err).expect("Should only have one owner")
            }
            DbErr::Query(RuntimeErr::Rusqlite(err)) => {
                Arc::into_inner(err).expect("Should only have one owner")
            }
            _ => RusqliteError::InvalidParameterName(err.to_string()),
        }),
    }
}
