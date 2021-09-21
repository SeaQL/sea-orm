use std::{pin::Pin, sync::Arc, task::Poll};

use futures::{Stream, TryStreamExt};

use sqlx::{pool::PoolConnection, Executor};

use crate::{sqlx_error_to_query_err, DbErr, QueryResult, Statement};

enum Connection {
    #[cfg(feature = "sqlx-mysql")]
    MySql(PoolConnection<sqlx::MySql>),
    #[cfg(feature = "sqlx-postgres")]
    Postgres(PoolConnection<sqlx::Postgres>),
    #[cfg(feature = "sqlx-sqlite")]
    Sqlite(PoolConnection<sqlx::Sqlite>),
}

pub struct QueryStream {
    stmt: Arc<Statement>,
    conn: Arc<Connection>,
    stream: Option<Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>>>>>,
}

#[cfg(feature = "sqlx-mysql")]
impl From<(PoolConnection<sqlx::MySql>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::MySql>, Statement)) -> Self {
        QueryStream {
            stmt: Arc::new(stmt),
            conn: Arc::new(Connection::MySql(conn)),
            stream: None
        }
    }
}

#[cfg(feature = "sqlx-postgres")]
impl From<(PoolConnection<sqlx::Postgres>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::Postgres>, Statement)) -> Self {
        QueryStream {
            stmt: Arc::new(stmt),
            conn: Arc::new(Connection::Postgres(conn)),
            stream: None
        }
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl From<(PoolConnection<sqlx::Sqlite>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::Sqlite>, Statement)) -> Self {
        QueryStream {
            stmt: Arc::new(stmt),
            conn: Arc::new(Connection::Sqlite(conn)),
            stream: None
        }
    }
}

impl std::fmt::Debug for QueryStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStream")
    }
}

impl QueryStream {
    fn get_conn(&mut self) -> Option<&'static mut Connection> {
        // this is safe since the connection is owned and the stream, that references the connection, is owned too, so they die tougheter
        unsafe { std::mem::transmute(Arc::get_mut(&mut self.conn)) }
    }
    fn get_stmt(&self) -> &'static Statement {
        // this is safe since the statement is owned and the stream, that references the statement, is owned too, so they die tougheter
        unsafe { std::mem::transmute(self.stmt.as_ref()) }
    }
    fn init(&mut self) {
        match self.get_conn() {
            #[cfg(feature = "sqlx-mysql")]
            Some(Connection::MySql(c)) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(self.get_stmt());
                self.stream = Some(Box::pin(
                    c.fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err)
                ));
            },
            #[cfg(feature = "sqlx-postgres")]
            Some(Connection::Postgres(c)) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(self.get_stmt());
                self.stream = Some(Box::pin(
                    c.fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err)
                ));
            },
            #[cfg(feature = "sqlx-sqlite")]
            Some(Connection::Sqlite(c)) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(self.get_stmt());
                self.stream = Some(Box::pin(
                    c.fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err)
                ));
            },
            _ => unreachable!(),
        }
    }
}

impl Stream for QueryStream {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.stream.is_none() {
            this.init();
        }
        if let Some(stream) = this.stream.as_mut() {
            stream.as_mut().poll_next(cx)
        }
        else {
            unreachable!();
        }
    }
}
