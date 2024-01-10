#![allow(missing_docs, unreachable_code, unused_variables)]

use tracing::instrument;

#[cfg(any(feature = "mock", feature = "proxy"))]
use std::sync::Arc;
use std::{pin::Pin, task::Poll};

use futures::Stream;
#[cfg(feature = "sqlx-dep")]
use futures::TryStreamExt;

#[cfg(feature = "sqlx-dep")]
use sqlx::{pool::PoolConnection, Executor};

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

#[cfg(feature = "sqlx-mysql")]
impl
    From<(
        PoolConnection<sqlx::MySql>,
        Statement,
        Option<crate::metric::Callback>,
    )> for QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            PoolConnection<sqlx::MySql>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        QueryStream::build(stmt, InnerConnection::MySql(conn), metric_callback)
    }
}

#[cfg(feature = "sqlx-postgres")]
impl
    From<(
        PoolConnection<sqlx::Postgres>,
        Statement,
        Option<crate::metric::Callback>,
    )> for QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            PoolConnection<sqlx::Postgres>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        QueryStream::build(stmt, InnerConnection::Postgres(conn), metric_callback)
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl
    From<(
        PoolConnection<sqlx::Sqlite>,
        Statement,
        Option<crate::metric::Callback>,
    )> for QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            PoolConnection<sqlx::Sqlite>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        QueryStream::build(stmt, InnerConnection::Sqlite(conn), metric_callback)
    }
}

#[cfg(feature = "mock")]
impl
    From<(
        Arc<crate::driver::MockDatabaseConnection>,
        Statement,
        Option<crate::metric::Callback>,
    )> for QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            Arc<crate::driver::MockDatabaseConnection>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        QueryStream::build(stmt, InnerConnection::Mock(conn), metric_callback)
    }
}

#[cfg(feature = "proxy")]
impl
    From<(
        Arc<crate::ProxyDatabaseConnection>,
        Statement,
        Option<crate::metric::Callback>,
    )> for QueryStream
{
    fn from(
        (conn, stmt, metric_callback): (
            Arc<crate::ProxyDatabaseConnection>,
            Statement,
            Option<crate::metric::Callback>,
        ),
    ) -> Self {
        QueryStream::build(stmt, InnerConnection::Proxy(conn), metric_callback)
    }
}

impl std::fmt::Debug for QueryStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QueryStream")
    }
}

impl QueryStream {
    #[instrument(level = "trace", skip(metric_callback))]
    fn build(
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
                    let _start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err);
                    let elapsed = _start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "sqlx-postgres")]
                InnerConnection::Postgres(c) => {
                    let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                    let _start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err);
                    let elapsed = _start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "sqlx-sqlite")]
                InnerConnection::Sqlite(c) => {
                    let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                    let _start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(sqlx_error_to_query_err);
                    let elapsed = _start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "mock")]
                InnerConnection::Mock(c) => {
                    let _start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c.fetch(stmt);
                    let elapsed = _start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[cfg(feature = "proxy")]
                InnerConnection::Proxy(c) => {
                    let _start = _metric_callback.is_some().then(std::time::SystemTime::now);
                    let stream = c.fetch(stmt);
                    let elapsed = _start.map(|s| s.elapsed().unwrap_or_default());
                    MetricStream::new(_metric_callback, stmt, elapsed, stream)
                }
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
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
        this.with_stream_mut(|stream| Pin::new(stream).poll_next(cx))
    }
}
