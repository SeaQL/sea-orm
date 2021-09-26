use std::{pin::Pin, task::Poll};

use futures::{Stream, TryStreamExt};

use sqlx::{pool::PoolConnection, Executor};

use crate::{DatabaseTransaction, DbErr, InnerConnection, QueryResult, Statement, sqlx_error_to_query_err};

#[ouroboros::self_referencing]
pub struct QueryStream<'a> {
    stmt: Statement,
    conn: InnerConnection<'a>,
    #[borrows(mut conn, stmt)]
    #[not_covariant]
    stream: Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + 'this>>,
}

#[cfg(feature = "sqlx-mysql")]
impl<'a> From<(PoolConnection<sqlx::MySql>, Statement)> for QueryStream<'a> {
    fn from((conn, stmt): (PoolConnection<sqlx::MySql>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::MySql(conn))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'a> From<(PoolConnection<sqlx::Postgres>, Statement)> for QueryStream<'a> {
    fn from((conn, stmt): (PoolConnection<sqlx::Postgres>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Postgres(conn))
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl<'a> From<(PoolConnection<sqlx::Sqlite>, Statement)> for QueryStream<'a> {
    fn from((conn, stmt): (PoolConnection<sqlx::Sqlite>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Sqlite(conn))
    }
}

#[cfg(feature = "mock")]
impl<'a> From<(&'a crate::MockDatabaseConnection, Statement)> for QueryStream<'a> {
    fn from((conn, stmt): (&'a crate::MockDatabaseConnection, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Mock(conn))
    }
}

impl<'a> From<(&'a DatabaseTransaction<'a>, Statement)> for QueryStream<'a> {
    fn from((conn, stmt): (&'a DatabaseTransaction<'a>, Statement)) -> Self {
        QueryStream::build(stmt, InnerConnection::Transaction(Box::new(conn)))
    }
}

impl<'a> std::fmt::Debug for QueryStream<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStream")
    }
}

impl<'a> QueryStream<'a> {
    fn build(stmt: Statement, conn: InnerConnection<'a>) -> QueryStream<'a> {
        QueryStreamBuilder {
            stmt,
            conn,
            stream_builder: |conn, stmt| {
                match conn {
                    #[cfg(feature = "sqlx-mysql")]
                    InnerConnection::MySql(c) => {
                        let query = crate::driver::sqlx_mysql::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(sqlx_error_to_query_err)
                        )
                    },
                    #[cfg(feature = "sqlx-postgres")]
                    InnerConnection::Postgres(c) => {
                        let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(sqlx_error_to_query_err)
                        )
                    },
                    #[cfg(feature = "sqlx-sqlite")]
                    InnerConnection::Sqlite(c) => {
                        let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(sqlx_error_to_query_err)
                        )
                    },
                    #[cfg(feature = "mock")]
                    InnerConnection::Mock(c) => {
                        c.fetch(stmt)
                    },
                    InnerConnection::Transaction(c) => {
                        c.fetch(stmt)
                    },
                }
            },
        }.build()
    }
}

impl<'a> Stream for QueryStream<'a> {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.with_stream_mut(|stream| {
            stream.as_mut().poll_next(cx)
        })
    }
}
