use crate::{
    debug_print, error::*, DatabaseConnection, DbBackend, ExecResult, ProxyDatabaseTrait,
    QueryResult, Statement,
};
use futures::Stream;
use std::{
    fmt::Debug,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tracing::instrument;

/// Defines a database driver for the [ProxyDatabase]
#[derive(Debug)]
pub struct ProxyDatabaseConnector;

/// Defines a connection for the [ProxyDatabase]
#[derive(Debug)]
pub struct ProxyDatabaseConnection {
    db_backend: DbBackend,
    proxy: Arc<Mutex<Box<dyn ProxyDatabaseTrait>>>,
}

impl ProxyDatabaseConnector {
    /// Check if the database URI given and the [DatabaseBackend](crate::DatabaseBackend) selected are the same
    #[allow(unused_variables)]
    pub fn accepts(string: &str) -> bool {
        // As this is a proxy database, it accepts any URI
        true
    }

    /// Connect to the [ProxyDatabase]
    #[allow(unused_variables)]
    #[instrument(level = "trace")]
    pub fn connect(
        db_type: DbBackend,
        func: Arc<Mutex<Box<dyn ProxyDatabaseTrait>>>,
    ) -> Result<DatabaseConnection, DbErr> {
        Ok(DatabaseConnection::ProxyDatabaseConnection(Arc::new(
            ProxyDatabaseConnection::new(db_type, func),
        )))
    }
}

impl ProxyDatabaseConnection {
    /// Create a connection to the [ProxyDatabase]
    pub fn new(db_backend: DbBackend, funcs: Arc<Mutex<Box<dyn ProxyDatabaseTrait>>>) -> Self {
        Self {
            db_backend,
            proxy: funcs.to_owned(),
        }
    }

    /// Get the [DatabaseBackend](crate::DatabaseBackend) being used by the [ProxyDatabase]
    pub fn get_database_backend(&self) -> DbBackend {
        self.db_backend
    }

    /// Execute the SQL statement in the [ProxyDatabase]
    #[instrument(level = "trace")]
    pub fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", statement);
        Ok(self
            .proxy
            .lock()
            .map_err(exec_err)?
            .execute(statement)?
            .into())
    }

    /// Return one [QueryResult] if the query was successful
    #[instrument(level = "trace")]
    pub fn query_one(&self, statement: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let result = self.proxy.lock().map_err(query_err)?.query(statement)?;

        if let Some(first) = result.first() {
            return Ok(Some(QueryResult {
                row: crate::QueryResultRow::Proxy(first.to_owned()),
            }));
        } else {
            return Ok(None);
        }
    }

    /// Return all [QueryResult]s if the query was successful
    #[instrument(level = "trace")]
    pub fn query_all(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let result = self.proxy.lock().map_err(query_err)?.query(statement)?;

        Ok(result
            .into_iter()
            .map(|row| QueryResult {
                row: crate::QueryResultRow::Proxy(row),
            })
            .collect())
    }

    /// Return [QueryResult]s  from a multi-query operation
    #[instrument(level = "trace")]
    pub fn fetch(
        &self,
        statement: &Statement,
    ) -> Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + Send>> {
        match self.query_all(statement.clone()) {
            Ok(v) => Box::pin(futures::stream::iter(v.into_iter().map(Ok))),
            Err(e) => Box::pin(futures::stream::iter(Some(Err(e)).into_iter())),
        }
    }

    /// Create a statement block  of SQL statements that execute together.
    #[instrument(level = "trace")]
    pub fn begin(&self) {
        self.proxy.lock().expect("Failed to acquire mocker").begin()
    }

    /// Commit a transaction atomically to the database
    #[instrument(level = "trace")]
    pub fn commit(&self) {
        self.proxy
            .lock()
            .expect("Failed to acquire mocker")
            .commit()
    }

    /// Roll back a faulty transaction
    #[instrument(level = "trace")]
    pub fn rollback(&self) {
        self.proxy
            .lock()
            .expect("Failed to acquire mocker")
            .rollback()
    }

    /// Checks if a connection to the database is still valid.
    pub fn ping(&self) -> Result<(), DbErr> {
        self.proxy.lock().map_err(query_err)?.ping()
    }
}
