use sqlx::{
    sqlite::{SqliteArguments, SqliteQueryResult, SqliteRow},
    Sqlite, SqlitePool,
};

sea_query::sea_query_driver_sqlite!();
use sea_query_driver_sqlite::bind_query;

use crate::{debug_print, error::*, executor::*, DatabaseConnection, Statement};

pub struct SqlxSqliteConnector;

pub struct SqlxSqlitePoolConnection {
    pool: SqlitePool,
}

impl SqlxSqliteConnector {
    pub fn accepts(string: &str) -> bool {
        string.starts_with("sqlite:")
    }

    pub async fn connect(string: &str) -> Result<DatabaseConnection, OrmError> {
        if let Ok(pool) = SqlitePool::connect(string).await {
            Ok(DatabaseConnection::SqlxSqlitePoolConnection(
                SqlxSqlitePoolConnection { pool },
            ))
        } else {
            Err(OrmError::Connection)
        }
    }
}

impl SqlxSqliteConnector {
    pub fn from_sqlx_sqlite_pool(pool: SqlitePool) -> DatabaseConnection {
        DatabaseConnection::SqlxSqlitePoolConnection(SqlxSqlitePoolConnection { pool })
    }
}

impl SqlxSqlitePoolConnection {
    pub async fn execute(&self, stmt: Statement) -> Result<ExecResult, OrmError> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(res) = query.execute(conn).await {
                return Ok(res.into());
            }
        }
        Err(OrmError::Execution)
    }

    pub async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, OrmError> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(row) = query.fetch_one(conn).await {
                Ok(Some(row.into()))
            } else {
                Ok(None)
            }
        } else {
            Err(OrmError::Query)
        }
    }

    pub async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, OrmError> {
        debug_print!("{}", stmt);

        let query = sqlx_query(&stmt);
        if let Ok(conn) = &mut self.pool.acquire().await {
            if let Ok(rows) = query.fetch_all(conn).await {
                return Ok(rows.into_iter().map(|r| r.into()).collect());
            }
        }
        Err(OrmError::Query)
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
