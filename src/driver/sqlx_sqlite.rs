use std::{future::Future, pin::Pin};

use sqlx::{
    sqlite::{SqliteArguments, SqliteConnectOptions, SqliteQueryResult, SqliteRow},
    Sqlite, SqlitePool,
};

sea_query::sea_query_driver_sqlite!();
use sea_query_driver_sqlite::bind_query;

use crate::{
    debug_print, error::*, executor::*, ConnectOptions, DatabaseConnection, DatabaseTransaction,
    QueryStream, Statement, TransactionError,
};

use super::sqlx_common::*;

#[derive(Debug)]
pub struct SqlxSqliteConnector;

#[derive(Debug, Clone)]
pub struct SqlxSqlitePoolConnection {
    pool: SqlitePool,
}

impl SqlxSqliteConnector {
    pub fn accepts(string: &str) -> bool {
        string.starts_with("sqlite:") && string.parse::<SqliteConnectOptions>().is_ok()
    }

    pub async fn connect(options: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
        let mut opt = options
            .url
            .parse::<SqliteConnectOptions>()
            .map_err(|e| DbErr::Conn(e.to_string()))?;
        if !options.sqlx_logging {
            use sqlx::ConnectOptions;
            opt.disable_statement_logging();
        }
        if let Ok(pool) = options
            .pool_options()
            .max_connections(1)
            .connect_with(opt)
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

    pub async fn stream(&self, stmt: Statement) -> Result<QueryStream, DbErr> {
        debug_print!("{}", stmt);

        if let Ok(conn) = self.pool.acquire().await {
            Ok(QueryStream::from((conn, stmt)))
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    pub async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        if let Ok(conn) = self.pool.acquire().await {
            DatabaseTransaction::new_sqlite(conn).await
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    pub async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(
                &'b DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>
            + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        if let Ok(conn) = self.pool.acquire().await {
            let transaction = DatabaseTransaction::new_sqlite(conn)
                .await
                .map_err(|e| TransactionError::Connection(e))?;
            transaction.run(callback).await
        } else {
            Err(TransactionError::Connection(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            )))
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

pub(crate) fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, Sqlite, SqliteArguments> {
    let mut query = sqlx::query(&stmt.sql);
    if let Some(values) = &stmt.values {
        query = bind_query(query, values);
    }
    query
}
