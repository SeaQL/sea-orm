use sqlx::{
    mysql::{MySqlArguments, MySqlQueryResult, MySqlRow},
    MySql, MySqlPool,
};

sea_query::sea_query_driver_mysql!();
use sea_query_driver_mysql::bind_query;

use crate::{debug_print, error::*, executor::*, DatabaseConnection, Statement};

use super::sqlx_common::*;

#[derive(Debug)]
pub struct SqlxMySqlConnector;

#[derive(Debug)]
pub struct SqlxMySqlPoolConnection {
    pool: MySqlPool,
}

impl SqlxMySqlConnector {
    pub fn accepts(string: &str) -> bool {
        string.starts_with("mysql://")
    }

    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        if let Ok(pool) = MySqlPool::connect(string).await {
            Ok(DatabaseConnection::SqlxMySqlPoolConnection(
                SqlxMySqlPoolConnection { pool },
            ))
        } else {
            Err(DbErr::Conn("Failed to connect.".to_owned()))
        }
    }
}

impl SqlxMySqlConnector {
    pub fn from_sqlx_mysql_pool(pool: MySqlPool) -> DatabaseConnection {
        DatabaseConnection::SqlxMySqlPoolConnection(SqlxMySqlPoolConnection { pool })
    }
}

impl SqlxMySqlPoolConnection {
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

impl From<MySqlRow> for QueryResult {
    fn from(row: MySqlRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxMySql(row),
        }
    }
}

impl From<MySqlQueryResult> for ExecResult {
    fn from(result: MySqlQueryResult) -> ExecResult {
        ExecResult {
            result: ExecResultHolder::SqlxMySql(result),
        }
    }
}

fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, MySql, MySqlArguments> {
    let mut query = sqlx::query(&stmt.sql);
    if let Some(values) = &stmt.values {
        query = bind_query(query, values);
    }
    query
}
