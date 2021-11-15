#![allow(missing_docs)]

use std::{pin::Pin, task::Poll};

#[cfg(feature = "mock")]
use std::sync::Arc;

use futures::Stream;
#[cfg(feature = "sqlx-dep")]
use futures::TryStreamExt;

#[cfg(feature = "sqlx-dep")]
use sqlx::{pool::PoolConnection, Executor};

use crate::{DbErr, InnerConnection, QueryResult, Statement};

/// Creates a stream from a [QueryResult]
#[ouroboros::self_referencing]
pub struct QueryStream {
    stmt: Statement,
    conn: InnerConnection,
    #[borrows(mut conn, stmt)]
    #[not_covariant]
    stream: Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + 'this>>,
}

#[cfg(feature = "sqlx-mysql")]
impl From<(PoolConnection<sqlx::MySql>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::MySql>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::MySql(conn))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl From<(PoolConnection<sqlx::Postgres>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::Postgres>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Postgres(conn))
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl From<(PoolConnection<sqlx::Sqlite>, Statement)> for QueryStream {
    fn from((conn, stmt): (PoolConnection<sqlx::Sqlite>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Sqlite(conn))
    }
}

#[cfg(feature = "mock")]
impl From<(Arc<crate::MockDatabaseConnection>, Statement)> for QueryStream {
    fn from((conn, stmt): (Arc<crate::MockDatabaseConnection>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Mock(conn))
    }
}

impl std::fmt::Debug for QueryStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStream")
    }
}

impl QueryStream {
    fn build(stmt: Statement, conn: InnerConnection) -> QueryStream {
        QueryStreamBuilder {
            stmt,
            conn,
            stream_builder: |conn, stmt| match conn {
                #[cfg(feature = "sqlx-mysql")]
                InnerConnection::MySql(c) => {
                    let query = crate::driver::sqlx_mysql::sqlx_query(stmt);
                    Box::pin(
                        c.fetch(query)
                            .map_ok(Into::into)
                            .map_err(crate::sqlx_error_to_query_err),
                    )
                }
                #[cfg(feature = "sqlx-postgres")]
                InnerConnection::Postgres(c) => {
                    let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                    Box::pin(
                        c.fetch(query)
                            .map_ok(Into::into)
                            .map_err(crate::sqlx_error_to_query_err),
                    )
                }
                #[cfg(feature = "sqlx-sqlite")]
                InnerConnection::Sqlite(c) => {
                    let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                    Box::pin(
                        c.fetch(query)
                            .map_ok(Into::into)
                            .map_err(crate::sqlx_error_to_query_err),
                    )
                }
                #[cfg(feature = "mock")]
                InnerConnection::Mock(c) => c.fetch(stmt),
            },
        }
        .build()
    }
}

impl Stream for QueryStream {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.with_stream_mut(|stream| stream.as_mut().poll_next(cx))
    }
}
