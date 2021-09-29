use std::{ops::DerefMut, pin::Pin, task::Poll};

use futures::Stream;
#[cfg(feature = "sqlx-dep")]
use futures::TryStreamExt;

#[cfg(feature = "sqlx-dep")]
use sqlx::Executor;

use futures::lock::MutexGuard;

use crate::{DbErr, InnerConnection, QueryResult, Statement};

#[ouroboros::self_referencing]
pub struct TransactionStream<'a> {
    stmt: Statement,
    conn: MutexGuard<'a, InnerConnection>,
    #[borrows(mut conn, stmt)]
    #[not_covariant]
    stream: Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + 'this>>,
}

impl<'a> std::fmt::Debug for TransactionStream<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TransactionStream")
    }
}

impl<'a> TransactionStream<'a> {
    pub(crate) async fn build(conn: MutexGuard<'a, InnerConnection>, stmt: Statement) -> TransactionStream<'a> {
        TransactionStreamAsyncBuilder {
            stmt,
            conn,
            stream_builder: |conn, stmt| Box::pin(async move {
                match conn.deref_mut() {
                    #[cfg(feature = "sqlx-mysql")]
                    InnerConnection::MySql(c) => {
                        let query = crate::driver::sqlx_mysql::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(crate::sqlx_error_to_query_err)
                        ) as Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>>>>
                    },
                    #[cfg(feature = "sqlx-postgres")]
                    InnerConnection::Postgres(c) => {
                        let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(crate::sqlx_error_to_query_err)
                        ) as Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>>>>
                    },
                    #[cfg(feature = "sqlx-sqlite")]
                    InnerConnection::Sqlite(c) => {
                        let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                        Box::pin(
                            c.fetch(query)
                                .map_ok(Into::into)
                                .map_err(crate::sqlx_error_to_query_err)
                        ) as Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>>>>
                    },
                    #[cfg(feature = "mock")]
                    InnerConnection::Mock(c) => {
                        c.fetch(stmt)
                    },
                }
            }),
        }.build().await
    }
}

impl<'a> Stream for TransactionStream<'a> {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.with_stream_mut(|stream| {
            stream.as_mut().poll_next(cx)
        })
    }
}
