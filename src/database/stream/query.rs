#![allow(missing_docs, unreachable_code, unused_variables)]

use futures_util::Stream;
use std::{pin::Pin, task::Poll};
use tracing::instrument;

#[cfg(feature = "sqlx-dep")]
use futures_util::TryStreamExt;

#[cfg(feature = "sqlx-dep")]
use sqlx::Executor;

use super::metric::MetricStream;
#[cfg(feature = "sqlx-dep")]
use crate::driver::*;
use crate::{DbErr, InnerConnection, QueryResult, Statement};

/// Creates a stream from a [QueryResult]
#[ouroboros::self_referencing]
pub struct QueryStream {
    stmt: Statement,
    conn: InnerConnection,
    metric_callback: Option<crate::metric::Callback>,
    #[borrows(mut conn, stmt, metric_callback)]
    #[not_covariant]
    stream: MetricStream<'this>,
}

impl std::fmt::Debug for QueryStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStream")
    }
}

impl QueryStream {
    #[allow(dead_code)]
    #[instrument(level = "trace", skip(metric_callback))]
    pub(crate) fn build(
        stmt: Statement,
        conn: InnerConnection,
        metric_callback: Option<crate::metric::Callback>,
    ) -> QueryStream {
        QueryStreamBuilder {
            stmt,
            conn,
            metric_callback,
            stream_builder: |conn, stmt, _metric_callback| match conn {
                #[cfg(feature = "sqlx-mysql")]
                InnerConnection::MySql(c) => {
                    let query = crate::driver::sqlx_mysql::sqlx_query(stmt);
                    let start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err);
                    let elapsed = start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "sqlx-postgres")]
                InnerConnection::Postgres(c) => {
                    let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                    let start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err);
                    let elapsed = start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "sqlx-sqlite")]
                InnerConnection::Sqlite(c) => {
                    let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                    let start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err);
                    let elapsed = start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "rusqlite")]
                InnerConnection::Rusqlite(conn) => {
                    use itertools::Either;
                    let start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = match conn.stream(stmt) {
                        Ok(rows) => Either::Left(rows.into_iter().map(Ok)),
                        Err(err) => Either::Right(std::iter::once(Err(err))),
                    };
                    let elapsed = start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "mock")]
                InnerConnection::Mock(c) => {
                    let start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c.fetch(stmt);
                    let elapsed = start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "proxy")]
                InnerConnection::Proxy(c) => {
                    let start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = futures_util::stream::once(async {
                        Err(DbErr::BackendNotSupported {
                            db: "Proxy",
                            ctx: "QueryStream",
                        })
                    });
                    let elapsed = start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                // D1 doesn't support streaming due to Send bound requirements
                // See db_connection.rs for stream_raw implementation
                #[cfg(feature = "d1")]
                InnerConnection::D1(_) => {
                    unreachable!("D1 streaming is not supported. Use query_all() instead.")
                }
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            },
        }
        .build()
    }
}

#[cfg(not(feature = "sync"))]
impl Stream for QueryStream {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.with_stream_mut(|stream| Pin::new(stream).poll_next(cx))
    }
}

#[cfg(feature = "sync")]
impl Iterator for QueryStream {
    type Item = Result<QueryResult, DbErr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.with_stream_mut(|stream| stream.next())
    }
}
