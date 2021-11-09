use regex::Regex;
use std::{future::Future, pin::Pin};

use sqlx::{
    mysql::{MySqlArguments, MySqlConnectOptions, MySqlQueryResult, MySqlRow},
    MySql, MySqlPool, Row,
};

sea_query::sea_query_driver_mysql!();
use sea_query_driver_mysql::bind_query;

use crate::{
    debug_print, error::*, executor::*, ConnectOptions, DatabaseConnection, DatabaseTransaction,
    DbBackend, QueryStream, Statement, TransactionError,
};

use super::sqlx_common::*;

/// Defines the [sqlx::mysql] connector
#[derive(Debug)]
pub struct SqlxMySqlConnector;

/// Defines a sqlx MySQL pool
#[derive(Debug, Clone)]
pub struct SqlxMySqlPoolConnection {
    pool: MySqlPool,
    pub(crate) support_returning: bool,
}

impl SqlxMySqlConnector {
    /// Check if the URI provided corresponds to `mysql://` for a MySQL database
    pub fn accepts(string: &str) -> bool {
        string.starts_with("mysql://") && string.parse::<MySqlConnectOptions>().is_ok()
    }

    /// Add configuration options for the MySQL database
    pub async fn connect(options: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
        let mut opt = options
            .url
            .parse::<MySqlConnectOptions>()
            .map_err(|e| DbErr::Conn(e.to_string()))?;
        if !options.sqlx_logging {
            use sqlx::ConnectOptions;
            opt.disable_statement_logging();
        }
        if let Ok(pool) = options.pool_options().connect_with(opt).await {
            into_db_connection(pool).await
        } else {
            Err(DbErr::Conn("Failed to connect.".to_owned()))
        }
    }
}

impl SqlxMySqlConnector {
    /// Instantiate a sqlx pool connection to a [DatabaseConnection]
    pub async fn from_sqlx_mysql_pool(pool: MySqlPool) -> Result<DatabaseConnection, DbErr> {
        into_db_connection(pool).await
    }
}

impl SqlxMySqlPoolConnection {
    /// Execute a [Statement] on a MySQL backend
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

    /// Get one result from a SQL query. Returns [Option::None] if no match was found
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

    /// Get the results of a query returning them as a Vec<[QueryResult]>
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

    /// Stream the results of executing a SQL query
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

    /// Bundle a set of SQL statements that execute together.
    pub async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        if let Ok(conn) = self.pool.acquire().await {
            DatabaseTransaction::new_mysql(conn, self.support_returning).await
        } else {
            Err(DbErr::Query(
                "Failed to acquire connection from pool.".to_owned(),
            ))
        }
    }

    /// Create a MySQL transaction
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
            let transaction = DatabaseTransaction::new_mysql(conn, self.support_returning)
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

pub(crate) fn sqlx_query(stmt: &Statement) -> sqlx::query::Query<'_, MySql, MySqlArguments> {
    let mut query = sqlx::query(&stmt.sql);
    if let Some(values) = &stmt.values {
        query = bind_query(query, values);
    }
    query
}

async fn into_db_connection(pool: MySqlPool) -> Result<DatabaseConnection, DbErr> {
    let support_returning = parse_support_returning(&pool).await?;
    Ok(DatabaseConnection::SqlxMySqlPoolConnection(
        SqlxMySqlPoolConnection {
            pool,
            support_returning,
        },
    ))
}

async fn parse_support_returning(pool: &MySqlPool) -> Result<bool, DbErr> {
    let stmt = Statement::from_string(
        DbBackend::MySql,
        r#"SHOW VARIABLES LIKE "version""#.to_owned(),
    );
    let query = sqlx_query(&stmt);
    let row = query
        .fetch_one(pool)
        .await
        .map_err(sqlx_error_to_query_err)?;
    let version: String = row.try_get("Value").map_err(sqlx_error_to_query_err)?;
    let support_returning = if !version.contains("MariaDB") {
        // This is MySQL
        // Not supported in all MySQL versions
        false
    } else {
        // This is MariaDB
        let regex = Regex::new(r"^(\d+)?.(\d+)?.(\*|\d+)").unwrap();
        let captures = regex.captures(&version).unwrap();
        macro_rules! parse_captures {
            ( $idx: expr ) => {
                captures.get($idx).map_or(0, |m| {
                    m.as_str()
                        .parse::<usize>()
                        .map_err(|e| DbErr::Conn(e.to_string()))
                        .unwrap()
                })
            };
        }
        let ver_major = parse_captures!(1);
        let ver_minor = parse_captures!(2);
        // Supported if it's MariaDB with version 10.5.0 or after
        ver_major >= 10 && ver_minor >= 5
    };
    debug_print!("db_version: {}", version);
    debug_print!("db_support_returning: {}", support_returning);
    Ok(support_returning)
}
