use async_trait::async_trait;
use sqlx::{mysql::MySqlRow, MySqlPool};

use sea_query::MysqlQueryBuilder;
sea_query::sea_query_driver_mysql!();
use sea_query_driver_mysql::bind_query;

use crate::{debug_print, executor::*, query::*};

pub struct SqlxMySqlExecutor {
    pool: MySqlPool,
}

#[async_trait]
impl Executor for SqlxMySqlExecutor {
    type QueryBuilder = MysqlQueryBuilder;

    async fn query_one(&self, stmt: Statement) -> Result<QueryResult, ExecErr> {
        debug_print!("{}, {:?}", sql, values);

        let query = bind_query(sqlx::query(&stmt.sql), &stmt.values);
        if let Ok(row) = query
            .fetch_one(&mut self.pool.acquire().await.unwrap())
            .await
        {
            Ok(row.into())
        } else {
            Err(ExecErr)
        }
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, ExecErr> {
        debug_print!("{}, {:?}", sql, values);

        let query = bind_query(sqlx::query(&stmt.sql), &stmt.values);
        if let Ok(rows) = query
            .fetch_all(&mut self.pool.acquire().await.unwrap())
            .await
        {
            Ok(rows.into_iter().map(|r| r.into()).collect())
        } else {
            Err(ExecErr)
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
