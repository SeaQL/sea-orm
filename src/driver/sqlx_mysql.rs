use sqlx::{
    mysql::{MySqlArguments, MySqlQueryResult, MySqlRow},
    MySql, MySqlPool,
};

sea_query::sea_query_driver_mysql!();
use sea_query_driver_mysql::bind_query;

use crate::{debug_print, error::*, executor::*, DatabaseConnection, Statement};

pub struct SqlxMySqlConnector;

pub struct SqlxMySqlPoolConnection {
    pool: MySqlPool,
}

impl SqlxMySqlConnector {
    pub fn accepts(string: &str) -> bool {
        string.starts_with("mysql://")
    }

    pub async fn connect(string: &str) -> Result<DatabaseConnection, SeaErr> {
        if let Ok(pool) = MySqlPool::connect(string).await {
            Ok(DatabaseConnection::SqlxMySqlPoolConnection(
                SqlxMySqlPoolConnection { pool },
            ))
        } else {
            Err(SeaErr::Connection)
        }
    }
}

impl SqlxMySqlConnector {
    pub fn from_sqlx_mysql_pool(pool: MySqlPool) -> DatabaseConnection {
        DatabaseConnection::SqlxMySqlPoolConnection(SqlxMySqlPoolConnection { pool })
    }
}

impl SqlxMySqlPoolConnection {
    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, SeaErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(res) = query.execute(conn).await {
                return Ok(res.into());
            }
        }
        Err(SeaErr::Execution)
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, SeaErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(row) = query.fetch_one(conn).await {
                Ok(Some(row.into()))
            } else {
                Ok(None)
            }
        } else {
            Err(SeaErr::Query)
        }
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, SeaErr> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(rows) = query.fetch_all(conn).await {
                return Ok(rows.into_iter().map(|r| r.into()).collect());
            }
        }
        Err(SeaErr::Query)
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
