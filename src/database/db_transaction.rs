use std::{cell::UnsafeCell, future::Future, pin::Pin};
use crate::{ConnectionTrait, DbBackend, DbErr, ExecResult, InnerConnection, QueryResult, QueryStream, Statement, debug_print};
use futures::{Stream, TryStreamExt};
#[cfg(feature = "sqlx-dep")]
use crate::{sqlx_error_to_exec_err, sqlx_error_to_query_err};
#[cfg(feature = "sqlx-dep")]
use sqlx::{pool::PoolConnection, Executor, TransactionManager};

// a Transaction is just a sugar for a connection where START TRANSACTION has been executed
pub struct DatabaseTransaction<'a> {
    // using Option we don't even need an "open" flag
    conn: Option<UnsafeCell<InnerConnection<'a>>>,
}

impl<'a> std::fmt::Debug for DatabaseTransaction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DatabaseTransaction")
    }
}

impl<'a> DatabaseTransaction<'a> {
    #[cfg(feature = "sqlx-mysql")]
    pub(crate) async fn new_mysql(inner: PoolConnection<sqlx::MySql>) -> Result<DatabaseTransaction<'a>, DbErr> {
        Self::build(InnerConnection::MySql(inner)).await
    }

    #[cfg(feature = "sqlx-postgres")]
    pub(crate) async fn new_postgres(inner: PoolConnection<sqlx::Postgres>) -> Result<DatabaseTransaction<'a>, DbErr> {
        Self::build(InnerConnection::Postgres(inner)).await
    }

    #[cfg(feature = "sqlx-sqlite")]
    pub(crate) async fn new_sqlite(inner: PoolConnection<sqlx::Sqlite>) -> Result<DatabaseTransaction<'a>, DbErr> {
        Self::build(InnerConnection::Sqlite(inner)).await
    }

    #[cfg(feature = "mock")]
    pub(crate) async fn new_mock(inner: &'a crate::MockDatabaseConnection) -> Result<DatabaseTransaction<'a>, DbErr> {
        Self::build(InnerConnection::Mock(inner)).await
    }

    async fn build(conn: InnerConnection<'a>) -> Result<DatabaseTransaction<'a>, DbErr> {
        let mut res = DatabaseTransaction {
            conn: Some(UnsafeCell::new(conn)),
        };
        match res.conn.as_mut().map(|c| c.get_mut()) {
            #[cfg(feature = "sqlx-mysql")]
            Some(InnerConnection::MySql(c)) => {
                <sqlx::MySql as sqlx::Database>::TransactionManager::begin(c).await.map_err(sqlx_error_to_query_err)?
            },
            #[cfg(feature = "sqlx-postgres")]
            Some(InnerConnection::Postgres(c)) => {
                <sqlx::Postgres as sqlx::Database>::TransactionManager::begin(c).await.map_err(sqlx_error_to_query_err)?
            },
            #[cfg(feature = "sqlx-sqlite")]
            Some(InnerConnection::Sqlite(c)) => {
                <sqlx::Sqlite as sqlx::Database>::TransactionManager::begin(c).await.map_err(sqlx_error_to_query_err)?
            },
            // should we do something for mocked connections?
            #[cfg(feature = "mock")]
            Some(InnerConnection::Mock(_)) => {},
            // nested transactions should already have been started
            Some(InnerConnection::Transaction(_)) => {},
            _ => unreachable!(),
        }
        Ok(res)
    }

    pub(crate) async fn run<F, T, E/*, Fut*/>(self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(&'b DatabaseTransaction<'a>) -> Pin<Box<dyn Future<Output = Result<T, E>> + 'b>>,
        // F: FnOnce(&DatabaseTransaction<'a>) -> Fut + Send,
        // Fut: Future<Output = Result<T, E>> + Send,
        // T: Send,
        E: std::error::Error,
    {
        let res = callback(&self).await.map_err(|e| TransactionError::Transaction(e));
        if res.is_ok() {
            self.commit().await.map_err(|e| TransactionError::Connection(e))?;
        }
        else {
            self.rollback().await.map_err(|e| TransactionError::Connection(e))?;
        }
        res
    }

    pub async fn commit(mut self) -> Result<(), DbErr> {
        match self.conn.take().map(|c| c.into_inner()) {
            #[cfg(feature = "sqlx-mysql")]
            Some(InnerConnection::MySql(ref mut c)) => {
                <sqlx::MySql as sqlx::Database>::TransactionManager::commit(c).await.map_err(sqlx_error_to_query_err)?
            },
            #[cfg(feature = "sqlx-postgres")]
            Some(InnerConnection::Postgres(ref mut c)) => {
                <sqlx::Postgres as sqlx::Database>::TransactionManager::commit(c).await.map_err(sqlx_error_to_query_err)?
            },
            #[cfg(feature = "sqlx-sqlite")]
            Some(InnerConnection::Sqlite(ref mut c)) => {
                <sqlx::Sqlite as sqlx::Database>::TransactionManager::commit(c).await.map_err(sqlx_error_to_query_err)?
            },
            Some(InnerConnection::Transaction(c)) => c.ref_commit().await?,
            //Should we do something for mocked &connections?
            #[cfg(feature = "mock")]
            Some(InnerConnection::Mock(_)) => {},
            _ => unreachable!(),
        }
        Ok(())
    }

    // non destructive commit
    fn ref_commit(&'a self) -> Pin<Box<dyn Future<Output=Result<(), DbErr>> + 'a>> {
        Box::pin(async move {
            if self.conn.is_some() {
                match self.get_conn() {
                    #[cfg(feature = "sqlx-mysql")]
                    InnerConnection::MySql(ref mut c) => {
                        <sqlx::MySql as sqlx::Database>::TransactionManager::commit(c).await.map_err(sqlx_error_to_query_err)?
                    },
                    #[cfg(feature = "sqlx-postgres")]
                    InnerConnection::Postgres(ref mut c) => {
                        <sqlx::Postgres as sqlx::Database>::TransactionManager::commit(c).await.map_err(sqlx_error_to_query_err)?
                    },
                    #[cfg(feature = "sqlx-sqlite")]
                    InnerConnection::Sqlite(ref mut c) => {
                        <sqlx::Sqlite as sqlx::Database>::TransactionManager::commit(c).await.map_err(sqlx_error_to_query_err)?
                    },
                    InnerConnection::Transaction(c) => c.ref_commit().await?,
                    //Should we do something for mocked &connections?
                    #[cfg(feature = "mock")]
                    InnerConnection::Mock(_) => {},
                }
            }
            Ok(())
        })
    }

    pub async fn rollback(mut self) -> Result<(), DbErr> {
        match self.conn.take().map(|c| c.into_inner()) {
            #[cfg(feature = "sqlx-mysql")]
            Some(InnerConnection::MySql(ref mut c)) => {
                <sqlx::MySql as sqlx::Database>::TransactionManager::rollback(c).await.map_err(sqlx_error_to_query_err)?
            },
            #[cfg(feature = "sqlx-postgres")]
            Some(InnerConnection::Postgres(ref mut c)) => {
                <sqlx::Postgres as sqlx::Database>::TransactionManager::rollback(c).await.map_err(sqlx_error_to_query_err)?
            },
            #[cfg(feature = "sqlx-sqlite")]
            Some(InnerConnection::Sqlite(ref mut c)) => {
                <sqlx::Sqlite as sqlx::Database>::TransactionManager::rollback(c).await.map_err(sqlx_error_to_query_err)?
            },
            Some(InnerConnection::Transaction(c)) => c.ref_rollback().await?,
            //Should we do something for mocked &connections?
            #[cfg(feature = "mock")]
            Some(InnerConnection::Mock(_)) => {},
            _ => unreachable!(),
        }
        Ok(())
    }

    // non destructive rollback
    fn ref_rollback(&'a self) -> Pin<Box<dyn Future<Output=Result<(), DbErr>> + 'a>> {
        Box::pin(async move {
            if self.conn.is_some() {
                match self.get_conn() {
                    #[cfg(feature = "sqlx-mysql")]
                    InnerConnection::MySql(ref mut c) => {
                        <sqlx::MySql as sqlx::Database>::TransactionManager::rollback(c).await.map_err(sqlx_error_to_query_err)?
                    },
                    #[cfg(feature = "sqlx-postgres")]
                    InnerConnection::Postgres(ref mut c) => {
                        <sqlx::Postgres as sqlx::Database>::TransactionManager::rollback(c).await.map_err(sqlx_error_to_query_err)?
                    },
                    #[cfg(feature = "sqlx-sqlite")]
                    InnerConnection::Sqlite(ref mut c) => {
                        <sqlx::Sqlite as sqlx::Database>::TransactionManager::rollback(c).await.map_err(sqlx_error_to_query_err)?
                    },
                    InnerConnection::Transaction(c) => c.ref_rollback().await?,
                    //Should we do something for mocked &connections?
                    #[cfg(feature = "mock")]
                    InnerConnection::Mock(_) => {},
                }
            }
            Ok(())
        })
    }

    pub(crate) fn fetch<'b>(&'b self, stmt: &'b Statement) -> Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + 'b>> {
        match self.get_conn() {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(inner) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(stmt);
                Box::pin(inner.fetch(query)
                    .map_ok(Into::into)
                    .map_err(sqlx_error_to_query_err))
            },
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(inner) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(stmt);
                Box::pin(inner.fetch(query)
                    .map_ok(Into::into)
                    .map_err(sqlx_error_to_query_err))
            },
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(inner) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(stmt);
                Box::pin(inner.fetch(query)
                    .map_ok(Into::into)
                    .map_err(sqlx_error_to_query_err))
            },
            InnerConnection::Transaction(inner) => {
                inner.fetch(stmt)
            },
            #[cfg(feature = "mock")]
            InnerConnection::Mock(inner) => {
                inner.fetch(stmt)
            },
        }
    }

    // the rollback is queued and will be performed on next async operation, like returning the connection to the pool
    fn start_rollback(&self) {
        if let Some(conn) = self.conn.as_ref() {
            match unsafe { &mut *conn.get() } {
                #[cfg(feature = "sqlx-mysql")]
                InnerConnection::MySql(c) => {
                    <sqlx::MySql as sqlx::Database>::TransactionManager::start_rollback(c);
                },
                #[cfg(feature = "sqlx-postgres")]
                InnerConnection::Postgres(c) => {
                    <sqlx::Postgres as sqlx::Database>::TransactionManager::start_rollback(c);
                },
                #[cfg(feature = "sqlx-sqlite")]
                InnerConnection::Sqlite(c) => {
                    <sqlx::Sqlite as sqlx::Database>::TransactionManager::start_rollback(c);
                },
                InnerConnection::Transaction(c) => {
                    c.start_rollback();
                }
                //Should we do something for mocked &connections?
                #[cfg(feature = "mock")]
                InnerConnection::Mock(_) => {},
            }
        }
    }

    fn get_conn(&self) -> &mut InnerConnection<'a> {
        unsafe { &mut *self.conn.as_ref().map(|c| c.get()).unwrap() }
    }
}

impl<'a> Drop for DatabaseTransaction<'a> {
    fn drop(&mut self) {
        self.start_rollback();
    }
}

// this is needed since sqlite connections aren't sync
// unsafe impl<'a> Sync for DatabaseTransaction<'a> {}

#[async_trait::async_trait(?Send)]
impl<'a> ConnectionTrait<'a> for DatabaseTransaction<'a> {
    fn get_database_backend(&self) -> DbBackend {
        match self.conn.as_ref().map(|c| unsafe { &*c.get() }) {
            #[cfg(feature = "sqlx-mysql")]
            Some(InnerConnection::MySql(_)) => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            Some(InnerConnection::Postgres(_)) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            Some(InnerConnection::Sqlite(_)) => DbBackend::Sqlite,
            #[cfg(feature = "mock")]
            Some(InnerConnection::Mock(c)) => c.get_database_backend(),
            Some(InnerConnection::Transaction(c)) => c.get_database_backend(),
            _ => unreachable!(),
        }
    }

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let _res = match self.get_conn() {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                query.execute(conn).await
                    .map(Into::into)
            },
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                query.execute(conn).await
                    .map(Into::into)
            },
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                query.execute(conn).await
                    .map(Into::into)
            },
            #[cfg(feature = "mock")]
            InnerConnection::Mock(conn) => return conn.execute(stmt),
            InnerConnection::Transaction(conn) => return conn.execute(stmt).await,
        };
        #[cfg(feature = "sqlx-dep")]
        _res.map_err(sqlx_error_to_exec_err)
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let _res = match self.get_conn() {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                query.fetch_one(conn).await
                    .map(|row| Some(row.into()))
            },
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                query.fetch_one(conn).await
                    .map(|row| Some(row.into()))
            },
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                query.fetch_one(conn).await
                    .map(|row| Some(row.into()))
            },
            #[cfg(feature = "mock")]
            InnerConnection::Mock(conn) => return conn.query_one(stmt),
            InnerConnection::Transaction(conn) => return conn.query_one(stmt).await,
        };
        #[cfg(feature = "sqlx-dep")]
        if let Err(sqlx::Error::RowNotFound) = _res {
            Ok(None)
        }
        else {
            _res.map_err(sqlx_error_to_query_err)
        }
    }

    async fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let _res = match self.get_conn() {
            #[cfg(feature = "sqlx-mysql")]
            InnerConnection::MySql(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                query.fetch_all(conn).await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
            },
            #[cfg(feature = "sqlx-postgres")]
            InnerConnection::Postgres(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                query.fetch_all(conn).await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
            },
            #[cfg(feature = "sqlx-sqlite")]
            InnerConnection::Sqlite(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                query.fetch_all(conn).await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
            },
            #[cfg(feature = "mock")]
            InnerConnection::Mock(conn) => return conn.query_all(stmt),
            InnerConnection::Transaction(conn) => return conn.query_all(stmt).await,
        };
        #[cfg(feature = "sqlx-dep")]
        _res.map_err(sqlx_error_to_query_err)
    }

    async fn stream(&'a self, stmt: Statement) -> Result<QueryStream<'a>, DbErr> {
        Ok(QueryStream::from((self, stmt)))
    }

    async fn begin(&'a self) -> Result<DatabaseTransaction<'a>, DbErr> {
        DatabaseTransaction::build(InnerConnection::Transaction(Box::new(self))).await
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E/*, Fut*/>(&'a self, _callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction<'a>) -> Pin<Box<dyn Future<Output = Result<T, E>> + 'c>>,
        // F: FnOnce(&DatabaseTransaction<'a>) -> Fut + Send,
        // Fut: Future<Output = Result<T, E>> + Send,
        // T: Send,
        E: std::error::Error,
    {
        let transaction = self.begin().await.map_err(|e| TransactionError::Connection(e))?;
        transaction.run(_callback).await
    }
}

#[derive(Debug)]
pub enum TransactionError<E>
where E: std::error::Error {
    Connection(DbErr),
    Transaction(E),
}

impl<E> std::fmt::Display for TransactionError<E>
where E: std::error::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::Connection(e) => std::fmt::Display::fmt(e, f),
            TransactionError::Transaction(e) => std::fmt::Display::fmt(e, f),
        }
    }
}

impl<E> std::error::Error for TransactionError<E>
where E: std::error::Error {}
