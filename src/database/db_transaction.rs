use std::{pin::Pin, future::Future};
use crate::{DbBackend, ConnectionTrait, DbErr, ExecResult, QueryResult, Statement, debug_print};
#[cfg(feature = "sqlx-dep")]
use crate::{sqlx_error_to_exec_err, sqlx_error_to_query_err};
#[cfg(feature = "sqlx-dep")]
use sqlx::Connection;

#[cfg(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite"))]
use futures::lock::Mutex;

#[derive(Debug)]
pub enum DatabaseTransaction<'a>  {
    #[cfg(feature = "sqlx-mysql")]
    SqlxMySqlTransaction(Mutex<sqlx::Transaction<'a, sqlx::MySql>>),
    #[cfg(feature = "sqlx-postgres")]
    SqlxPostgresTransaction(Mutex<sqlx::Transaction<'a, sqlx::Postgres>>),
    #[cfg(feature = "sqlx-sqlite")]
    SqlxSqliteTransaction(Mutex<sqlx::Transaction<'a, sqlx::Sqlite>>),
    #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
    None(&'a ()),
}

#[cfg(feature = "sqlx-mysql")]
impl<'a> From<sqlx::Transaction<'a, sqlx::MySql>> for DatabaseTransaction<'a> {
    fn from(inner: sqlx::Transaction<'a, sqlx::MySql>) -> Self {
        DatabaseTransaction::SqlxMySqlTransaction(Mutex::new(inner))
    }
}

#[cfg(feature = "sqlx-postgres")]
impl<'a> From<sqlx::Transaction<'a, sqlx::Postgres>> for DatabaseTransaction<'a> {
    fn from(inner: sqlx::Transaction<'a, sqlx::Postgres>) -> Self {
        DatabaseTransaction::SqlxPostgresTransaction(Mutex::new(inner))
    }
}

#[cfg(feature = "sqlx-sqlite")]
impl<'a> From<sqlx::Transaction<'a, sqlx::Sqlite>> for DatabaseTransaction<'a> {
    fn from(inner: sqlx::Transaction<'a, sqlx::Sqlite>) -> Self {
        DatabaseTransaction::SqlxSqliteTransaction(Mutex::new(inner))
    }
}

#[allow(dead_code)]
impl<'a> DatabaseTransaction<'a> {
    pub(crate) async fn run<F, T, E>(self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(&'b DatabaseTransaction<'a>) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>> + Send + Sync,
        T: Send,
        E: std::error::Error + Send,
    {
        let res = callback(&self).await.map_err(|e| TransactionError::Transaction(e));
        if res.is_ok() {
            self.commit().await?;
        }
        else {
            self.rollback().await?;
        }
        res
    }

    async fn commit<E>(self) -> Result<(), TransactionError<E>>
    where E: std::error::Error {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(inner) => {
                let transaction = inner.into_inner();
                transaction.commit().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))
            },
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(inner) => {
                let transaction = inner.into_inner();
                transaction.commit().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))
            },
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(inner) => {
                let transaction = inner.into_inner();
                transaction.commit().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))
            },
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
        }
    }

    async fn rollback<E>(self) -> Result<(), TransactionError<E>>
    where E: std::error::Error {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(inner) => {
                let transaction = inner.into_inner();
                transaction.rollback().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))
            },
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(inner) => {
                let transaction = inner.into_inner();
                transaction.rollback().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))
            },
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(inner) => {
                let transaction = inner.into_inner();
                transaction.rollback().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))
            },
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
        }
    }
}

#[async_trait::async_trait]
impl<'a> ConnectionTrait for DatabaseTransaction<'a> {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(_) => DbBackend::MySql,
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(_) => DbBackend::Postgres,
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(_) => DbBackend::Sqlite,
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
        }
    }

    async fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let _res = match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.execute(&mut *conn).await
                    .map(Into::into)
            },
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.execute(&mut *conn).await
                    .map(Into::into)
            },
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.execute(&mut *conn).await
                    .map(Into::into)
            },
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
        };
        #[cfg(feature = "sqlx-dep")]
        _res.map_err(sqlx_error_to_exec_err)
    }

    async fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let _res = match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.fetch_one(&mut *conn).await
                    .map(|row| Some(row.into()))
            },
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.fetch_one(&mut *conn).await
                    .map(|row| Some(row.into()))
            },
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.fetch_one(&mut *conn).await
                    .map(|row| Some(row.into()))
            },
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
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

        let _res = match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(conn) => {
                let query = crate::driver::sqlx_mysql::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.fetch_all(&mut *conn).await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
            },
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(conn) => {
                let query = crate::driver::sqlx_postgres::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.fetch_all(&mut *conn).await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
            },
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(conn) => {
                let query = crate::driver::sqlx_sqlite::sqlx_query(&stmt);
                let mut conn = conn.lock().await;
                query.fetch_all(&mut *conn).await
                    .map(|rows| rows.into_iter().map(|r| r.into()).collect())
            },
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
        };
        #[cfg(feature = "sqlx-dep")]
        _res.map_err(sqlx_error_to_query_err)
    }

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    async fn transaction<F, T, E>(&self, _callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c DatabaseTransaction<'_>) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>> + Send + Sync,
        T: Send,
        E: std::error::Error + Send,
    {
        match self {
            #[cfg(feature = "sqlx-mysql")]
            DatabaseTransaction::SqlxMySqlTransaction(conn) => {
                let mut conn = conn.lock().await;
                let transaction = DatabaseTransaction::from(conn.begin().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))?);
                transaction.run(_callback).await
            },
            #[cfg(feature = "sqlx-postgres")]
            DatabaseTransaction::SqlxPostgresTransaction(conn) => {
                let mut conn = conn.lock().await;
                let transaction = DatabaseTransaction::from(conn.begin().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))?);
                transaction.run(_callback).await
            },
            #[cfg(feature = "sqlx-sqlite")]
            DatabaseTransaction::SqlxSqliteTransaction(conn) => {
                let mut conn = conn.lock().await;
                let transaction = DatabaseTransaction::from(conn.begin().await.map_err(|e| TransactionError::Connection(DbErr::Query(e.to_string())))?);
                transaction.run(_callback).await
            },
            #[cfg(not(any(feature = "sqlx-mysql", feature = "sqlx-postgres", feature = "sqlx-sqlite")))]
            _ => unimplemented!(),
        }
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
