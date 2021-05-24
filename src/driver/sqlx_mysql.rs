use async_trait::async_trait;
use sqlx::{mysql::MySqlRow, MySqlPool};

sea_query::sea_query_driver_mysql!();
use sea_query_driver_mysql::bind_query;

use crate::{connector::*, debug_print, query::*, Statement, DatabaseConnection};

pub struct SqlxMySqlConnector;

pub struct SqlxMySqlPoolConnection {
    pool: MySqlPool,
}

#[async_trait]
impl Connector for SqlxMySqlConnector {
    fn accepts(string: &str) -> bool {
        string.starts_with("mysql://")
    }

    async fn connect(string: &str) -> Result<DatabaseConnection, ConnectionErr> {
        if let Ok(conn) = MySqlPool::connect(string).await {
            Ok(DatabaseConnection::SqlxMySqlPoolConnection(
                SqlxMySqlPoolConnection { pool: conn },
            ))
        } else {
            Err(ConnectionErr)
        }
    }
}

#[async_trait]
impl Connection for &SqlxMySqlPoolConnection {
    async fn query_one(&self, stmt: Statement) -> Result<QueryResult, QueryErr> {
        debug_print!("{}", stmt);

        let query = bind_query(sqlx::query(&stmt.sql), &stmt.values);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(row) = query.fetch_one(conn).await {
                return Ok(row.into());
            }
        }
        Err(QueryErr)
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, QueryErr> {
        debug_print!("{}", stmt);

        let query = bind_query(sqlx::query(&stmt.sql), &stmt.values);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(rows) = query.fetch_all(conn).await {
                return Ok(rows.into_iter().map(|r| r.into()).collect());
            }
        }
        Err(QueryErr)
    }
}

impl From<MySqlRow> for QueryResult {
    fn from(row: MySqlRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::SqlxMySql(row),
        }
    }
}
