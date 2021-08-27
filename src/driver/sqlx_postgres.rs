use sqlx::{
    postgres::{PgArguments, PgQueryResult, PgRow},
    PgPool, Postgres,
};

sea_query::sea_query_driver_postgres!();
use sea_query_driver_postgres::bind_query;

use crate::{debug_print, error::*, executor::*, DatabaseConnection, Statement};

use super::sqlx_common::*;

pub struct SqlxPostgresConnector;

pub struct SqlxPostgresPoolConnection {
    pool: PgPool,
}

impl SqlxPostgresConnector {
    pub fn accepts(string: &str) -> bool {
        string.starts_with("postgres://")
    }

    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        if let Ok(pool) = PgPool::connect(string).await {
            Ok(DatabaseConnection::SqlxPostgresPoolConnection(
                SqlxPostgresPoolConnection { pool },
            ))
        } else {
            Err(DbErr::Conn("Failed to connect.".to_owned()))
        }
    }
}

impl SqlxPostgresConnector {
    pub fn from_sqlx_postgres_pool(pool: PgPool) -> DatabaseConnection {
        DatabaseConnection::SqlxPostgresPoolConnection(SqlxPostgresPoolConnection { pool })
    }
}

impl SqlxPostgresPoolConnection {
    pub async fn execute<T>(&self, stmt: Statement) -> Result<ExecResult<T>, DbErr> {
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

impl From<PgRow> for QueryResult {
    fn from(row: PgRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxPostgres(row),
        }
    }
}

impl<T> From<PgQueryResult> for ExecResult<T> {
    fn from(result: PgQueryResult) -> ExecResult<T> {
        ExecResult {
            result: ExecResultHolder {
                last_insert_id: None,
                rows_affected: result.rows_affected(),
            },
        }
    }
}

pub(crate) fn query_result_into_exec_result<T>(res: QueryResult) -> Result<ExecResult<T>, DbErr>
where
    T: TryGetable,
{
    let last_insert_id: T = res.try_get("", "last_insert_id")?;
    Ok(ExecResult {
        result: ExecResultHolder {
            last_insert_id: Some(last_insert_id),
            rows_affected: 0,
        },
    })
}

fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, Postgres, PgArguments> {
    let mut query = sqlx::query(&stmt.sql);
    if let Some(values) = &stmt.values {
        query = bind_query(query, values);
    }
    query
}
