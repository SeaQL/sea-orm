use std::{pin::Pin, task::Poll};

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

#[ouroboros::self_referencing]
pub struct QueryStream {
    stmt: Statement,
    conn: Connection,
    #[borrows(mut conn, stmt)]
    #[not_covariant]
    stream: Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + 'this>>,
}

#[cfg(feature = "sqlx-mysql")]
impl From<(PoolConnection<sqlx::MySql>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::MySql>, Statement)) -> Self {
        QueryStream::build(stmt, Connection::MySql(conn))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl From<(PoolConnection<sqlx::Postgres>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::Postgres>, Statement)) -> Self {
        QueryStream::build(stmt, Connection::Postgres(conn))
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl From<(PoolConnection<sqlx::Sqlite>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::Sqlite>, Statement)) -> Self {
        QueryStream::build(stmt, Connection::Sqlite(conn))
    }
}

impl std::fmt::Debug for QueryStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStream")
    }
}

impl QueryStream {
    fn build(stmt: Statement, conn: Connection) -> Self {
        QueryStreamBuilder {
            stmt,
            conn,
            stream_builder: |conn, stmt| {
                match conn {
                    #[cfg(feature = "sqlx-mysql")]
                    Connection::MySql(c) => {
                        let query = crate::driver::sqlx_mysql::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(sqlx_error_to_query_err)
                        )
                    },
                    #[cfg(feature = "sqlx-postgres")]
                    Connection::Postgres(c) => {
                        let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(sqlx_error_to_query_err)
                        )
                    },
                    #[cfg(feature = "sqlx-sqlite")]
                    Connection::Sqlite(c) => {
                        let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(sqlx_error_to_query_err)
                        )
                    },
                }
            },
        }.build()
    }
}

impl Stream for QueryStream {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.with_stream_mut(|stream| {
            stream.as_mut().poll_next(cx)
        })
    }
}
