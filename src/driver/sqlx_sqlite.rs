use sqlx::{
    sqlite::{SqliteArguments, SqlitePoolOptions, SqliteQueryResult, SqliteRow},
    Sqlite, SqlitePool,
};

sea_query::sea_query_driver_sqlite!();
use sea_query_driver_sqlite::bind_query;

use crate::{debug_print, error::*, executor::*, DatabaseConnection, DbBackend, Statement};

use super::sqlx_common::*;

#[derive(Debug)]
pub struct SqlxSqliteConnector;

#[derive(Debug, Clone)]
pub struct SqlxSqlitePoolConnection {
    pool: SqlitePool,
}

impl SqlxSqliteConnector {
    pub fn accepts(string: &str) -> bool {
        DbBackend::Sqlite.is_prefix_of(string)
    }

    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        if let Ok(pool) = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(string)
            .await
        {
            Ok(DatabaseConnection::SqlxSqlitePoolConnection(
                SqlxSqlitePoolConnection { pool },
            ))
        } else {
            Err(DbErr::Conn("Failed to connect.".to_owned()))
        }
    }
}

impl SqlxSqliteConnector {
    pub fn from_sqlx_sqlite_pool(pool: SqlitePool) -> DatabaseConnection {
        DatabaseConnection::SqlxSqlitePoolConnection(SqlxSqlitePoolConnection { pool })
    }
}

impl SqlxSqlitePoolConnection {
    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            match query.execute(conn).await {
                Ok(res) => Ok(res.into()),
                Err(err) => Err(sqlx_error_to_exec_err(err)),
            }
        } else {
            Err(DbErr::Exec(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            match query.fetch_one(conn).await {
                Ok(row) => Ok(Some(row.into())),
                Err(err) => match err {
                    sqlx::Error::RowNotFound => Ok(None),
                    _ => Err(DbErr::Query(err.to_string())),
                },
            }
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            match query.fetch_all(conn).await {
                Ok(rows) => Ok(rows.into_iter().map(|r| r.into()).collect()),
                Err(err) => Err(sqlx_error_to_query_err(err)),
            }
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }
}

impl From<SqliteRow> for QueryResult {
    fn from(row: SqliteRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxSqlite(row),
        }
    }
}

impl From<SqliteQueryResult> for ExecResult {
    fn from(result: SqliteQueryResult) -> ExecResult {
        ExecResult {
            result: ExecResultHolder::SqlxSqlite(result),
        }
    }
}

fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, Sqlite, SqliteArguments> {
    let mut query = sqlx::query(&stmt.sql);
    if let Some(values) = &stmt.values {
        query = bind_query(query, values);
    }
    query
}
