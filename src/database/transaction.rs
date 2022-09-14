use crate::{
    debug_print, ConnectionTrait, DbBackend, DbErr, ExecResult, InnerConnection, QueryResult,
    Statement, StreamTrait, TransactionStream, TransactionTrait,
};
use futures::lock::Mutex;
#[cfg(feature = "sqlx-dep")]
use sqlx::{pool::PoolConnection, TransactionManager};
use std::{future::Future, pin::Pin, sync::Arc};
use tracing::instrument;

// a Transaction is just a sugar for a connection where START TRANSACTION has been executed
/// Defines a database transaction, whether it is an open transaction and the type of
/// backend to use
pub struct DatabaseTransaction {
    conn: Arc<Mutex<InnerConnection>>,
    backend: DbBackend,
    open: bool,
    metric_callback: Option<crate::metric::Callback>,
}

impl std::fmt::Debug for DatabaseTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl DatabaseTransaction {
    #[cfg(feature = "sqlx-mysql")]
    pub(crate) async fn new_mysql(
        inner: PoolConnection<sqlx::MySql>,
        metric_callback: Option<crate::metric::Callback>,
    ) -> Result<DatabaseTransaction, DbErr> {
        Self::begin(
            Arc::new(Mutex::new(InnerConnection::MySql(inner))),
            DbBackend::MySql,
            metric_callback,
        )
        .await
    }

    #[cfg(feature = "sqlx-postgres")]
    pub(crate) async fn new_postgres(
        inner: PoolConnection<sqlx::Postgres>,
        metric_callback: Option<crate::metric::Callback>,
    ) -> Result<DatabaseTransaction, DbErr> {
        Self::begin(
            Arc::new(Mutex::new(InnerConnection::Postgres(inner))),
            DbBackend::Postgres,
            metric_callback,
        )
        .await
    }

    #[cfg(feature = "sqlx-sqlite")]
    pub(crate) async fn new_sqlite(
        inner: PoolConnection<sqlx::Sqlite>,
        metric_callback: Option<crate::metric::Callback>,
    ) -> Result<DatabaseTransaction, DbErr> {
        Self::begin(
            Arc::new(Mutex::new(InnerConnection::Sqlite(inner))),
            DbBackend::Sqlite,
            metric_callback,
        )
        .await
    }

    #[cfg(feature = "mock")]
    pub(crate) async fn new_mock(
        inner: Arc<crate::MockDatabaseConnection>,
        metric_callback: Option<crate::metric::Callback>,
    ) -> Result<DatabaseTransaction, DbErr> {
        let backend = inner.get_database_backend();
        Self::begin(
            Arc::new(Mutex::new(InnerConnection::Mock(inner))),
            backend,
            metric_callback,
        )
        .await
    }

    #[instrument(level = "trace", skip(metric_callback))]
    #[allow(unreachable_code)]
    async fn begin(
        conn: Arc<Mutex<InnerConnection>>,
        backend: DbBackend,
        metric_callback: Option<crate::metric::Callback>,
    ) -> Result<DatabaseTransaction, DbErr> {
        let res = DatabaseTransaction {
            conn,
            backend,
            open: true,
            metric_callback,
        };
        match *res.conn.lock().await {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(ref mut c) => {
                <sqlx::MySql as sqlx::Database>::TransactionManager::begin(c)
                    .await
                    .map_err(|e| backend.map_query_err(e))?
            }
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(ref mut c) => {
                <sqlx::Postgres as sqlx::Database>::TransactionManager::begin(c)
                    .await
                    .map_err(|e| backend.map_query_err(e))?
            }
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(ref mut c) => {
                <sqlx::Sqlite as sqlx::Database>::TransactionManager::begin(c)
                    .await
                    .map_err(|e| backend.map_query_err(e))?
            }
            #[cfg(feature = "mock")]
            InnerConnection::Mock(ref mut c) => {
                c.begin();
            }
        }
        Ok(res)
    }

    /// Runs a transaction to completion returning an rolling back the transaction on
    /// encountering an error if it fails
    #[instrument(level = "trace", skip(callback))]
    pub(crate) async fn run<F, T, E>(self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(
                &'b DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>
            + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        let res = callback(&self).await.map_err(TransactionError::Transaction);
        if res.is_ok() {
            self.commit().await.map_err(TransactionError::Connection)?;
        } else {
            self.rollback()
                .await
                .map_err(TransactionError::Connection)?;
        }
        res
    }

    /// Commit a transaction atomically
    #[instrument(level = "trace")]
    #[allow(unreachable_code)]
    pub async fn commit(mut self) -> Result<(), DbErr> {
        self.open = false;
        match *self.conn.lock().await {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(ref mut c) => {
                <sqlx::MySql as sqlx::Database>::TransactionManager::commit(c)
                    .await
                    .map_err(|e| self.backend.map_query_err(e))?
            }
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(ref mut c) => {
                <sqlx::Postgres as sqlx::Database>::TransactionManager::commit(c)
                    .await
                    .map_err(|e| self.backend.map_query_err(e))?
            }
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(ref mut c) => {
                <sqlx::Sqlite as sqlx::Database>::TransactionManager::commit(c)
                    .await
                    .map_err(|e| self.backend.map_query_err(e))?
            }
            #[cfg(feature = "mock")]
            InnerConnection::Mock(ref mut c) => {
                c.commit();
            }
        }
        Ok(())
    }

    /// rolls back a transaction in case error are encountered during the operation
    #[instrument(level = "trace")]
    #[allow(unreachable_code)]
    pub async fn rollback(mut self) -> Result<(), DbErr> {
        self.open = false;
        match *self.conn.lock().await {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(ref mut c) => {
                <sqlx::MySql as sqlx::Database>::TransactionManager::rollback(c)
                    .await
                    .map_err(|e| self.backend.map_query_err(e))?
            }
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(ref mut c) => {
                <sqlx::Postgres as sqlx::Database>::TransactionManager::rollback(c)
                    .await
                    .map_err(|e| self.backend.map_query_err(e))?
            }
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(ref mut c) => {
                <sqlx::Sqlite as sqlx::Database>::TransactionManager::rollback(c)
                    .await
                    .map_err(|e| self.backend.map_query_err(e))?
            }
            #[cfg(feature = "mock")]
            InnerConnection::Mock(ref mut c) => {
                c.rollback();
            }
        }
        Ok(())
    }

    // the rollback is queued and will be performed on next async operation, like returning the connection to the pool
    #[instrument(level = "trace")]
    fn start_rollback(&mut self) {
        if self.open {
            if let Some(mut conn) = self.conn.try_lock() {
                match &mut *conn {
                    #[cfg(feature = "sqlx-mysql")]
                    InnerConnection::MySql(c) => {
                        <sqlx::MySql as sqlx::Database>::TransactionManager::start_rollback(c);
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    InnerConnection::Postgres(c) => {
                        <sqlx::Postgres as sqlx::Database>::TransactionManager::start_rollback(c);
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    InnerConnection::Sqlite(c) => {
                        <sqlx::Sqlite as sqlx::Database>::TransactionManager::start_rollback(c);
                    }
                    #[cfg(feature = "mock")]
                    InnerConnection::Mock(c) => {
                        c.rollback();
                    }
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            } else {
                //this should never happen
                panic!("Dropping a locked Transaction");
            }
        }
    }

    #[cfg(feature = "sqlx-dep")]
    fn map_err_ignore_not_found<T: std::fmt::Debug>(
        err: Result<Option<T>, sqlx::Error>,
        backend: DbBackend,
    ) -> Result<Option<T>, DbErr> {
        if let Err(sqlx::Error::RowNotFound) = err {
            Ok(None)
        } else {
            err.map_err(|e| backend.map_query_err(e))
        }
    }
}

impl Drop for DatabaseTransaction {
    fn drop(&mut self) {
        self.start_rollback();
    }
}

#[async_trait::async_trait]
impl ConnectionTrait for DatabaseTransaction {
    fn get_database_backend(&self) -> DbBackend {
        // this way we don't need to lock
        self.backend
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        match &mut *self.conn.lock().await {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                crate::metric::metric!(self.metric_callback, &stmt, {
                    query.execute(conn).await.map(Into::into)
                })
                .map_err(|e| self.backend.map_exec_err(e))
            }
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                crate::metric::metric!(self.metric_callback, &stmt, {
                    query.execute(conn).await.map(Into::into)
                })
                .map_err(|e| self.backend.map_exec_err(e))
            }
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                crate::metric::metric!(self.metric_callback, &stmt, {
                    query.execute(conn).await.map(Into::into)
                })
                .map_err(|e| self.backend.map_exec_err(e))
            }
            #[cfg(feature = "mock")]
            InnerConnection::Mock(conn) => return conn.execute(stmt),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        match &mut *self.conn.lock().await {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                Self::map_err_ignore_not_found(
                    query.fetch_one(conn).await.map(|row| Some(row.into())),
                    self.backend,
                )
            }
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                Self::map_err_ignore_not_found(
                    query.fetch_one(conn).await.map(|row| Some(row.into())),
                    self.backend,
                )
            }
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                Self::map_err_ignore_not_found(
                    query.fetch_one(conn).await.map(|row| Some(row.into())),
                    self.backend,
                )
            }
            #[cfg(feature = "mock")]
            InnerConnection::Mock(conn) => return conn.query_one(stmt),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    #[instrument(level = "trace")]
    #[allow(unused_variables)]
    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        match &mut *self.conn.lock().await {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                query
                    .fetch_all(conn)
                    .await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
                    .map_err(|e| self.backend.map_query_err(e))
            }
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                query
                    .fetch_all(conn)
                    .await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
                    .map_err(|e| self.backend.map_query_err(e))
            }
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                query
                    .fetch_all(conn)
                    .await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
                    .map_err(|e| self.backend.map_query_err(e))
            }
            #[cfg(feature = "mock")]
            InnerConnection::Mock(conn) => return conn.query_all(stmt),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl<'a> StreamTrait<'a> for DatabaseTransaction {
    type Stream = TransactionStream<'a>;

    #[instrument(level = "trace")]
    fn stream(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream, DbErr>> + 'a + Send>> {
        Box::pin(async move {
            let conn = self.conn.lock().await;
            Ok(crate::TransactionStream::build(
                conn,
                stmt,
                self.metric_callback.clone(),
            ))
        })
    }
}

#[async_trait::async_trait]
impl TransactionTrait for DatabaseTransaction {
    #[instrument(level = "trace")]
    async fn begin(&self) -> Result<DatabaseTransaction, DbErr> {
        DatabaseTransaction::begin(
            Arc::clone(&self.conn),
            self.backend,
            self.metric_callback.clone(),
        )
        .await
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    #[instrument(level = "trace", skip(_callback))]
    async fn transaction<F, T, E>(&self, _callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c DatabaseTransaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::error::Error + Send,
    {
        let transaction = self.begin().await.map_err(TransactionError::Connection)?;
        transaction.run(_callback).await
    }
}

/// Defines errors for handling transaction failures
#[derive(Debug)]
pub enum TransactionError<E>
where
    E: std::error::Error,
{
    /// A Database connection error
    Connection(DbErr),
    /// An error occurring when doing database transactions
    Transaction(E),
}

impl<E> std::fmt::Display for TransactionError<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::Connection(e) => std::fmt::Display::fmt(e, f),
            TransactionError::Transaction(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl<E> std::error::Error for TransactionError<E> where E: std::error::Error {}
