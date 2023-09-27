use crate::{
    debug_print, error::*, DatabaseConnection, DbBackend, ExecResult, ProxyDatabase,
    ProxyDatabaseFuncTrait, QueryResult, Statement,
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
    proxy: Mutex<Box<dyn ProxyDatabaseTrait>>,
}

/// A Trait for any type wanting to perform operations on the [ProxyDatabase]
pub trait ProxyDatabaseTrait: Send + Debug {
    /// Execute a statement in the [ProxyDatabase]
    fn execute(&mut self, stmt: Statement) -> Result<ExecResult, DbErr>;

    /// Execute a SQL query in the [ProxyDatabase]
    fn query(&mut self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    /// Create a transaction that can be committed atomically
    fn begin(&mut self);

    /// Commit a successful transaction atomically into the [ProxyDatabase]
    fn commit(&mut self);

    /// Roll back a transaction since errors were encountered
    fn rollback(&mut self);

    /// Get the backend being used in the [ProxyDatabase]
    fn get_database_backend(&self) -> DbBackend;

    /// Ping the [ProxyDatabase]
    fn ping(&self) -> Result<(), DbErr>;
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
        func: Arc<dyn ProxyDatabaseFuncTrait>,
    ) -> Result<DatabaseConnection, DbErr> {
        Ok(DatabaseConnection::ProxyDatabaseConnection(Arc::new(
            ProxyDatabaseConnection::new(ProxyDatabase::new(db_type, func)),
        )))
    }
}

impl ProxyDatabaseConnection {
    /// Create a connection to the [ProxyDatabase]
    pub fn new<M: 'static>(m: M) -> Self
    where
        M: ProxyDatabaseTrait,
    {
        Self {
            proxy: Mutex::new(Box::new(m)),
        }
    }

    /// Get the [DatabaseBackend](crate::DatabaseBackend) being used by the [ProxyDatabase]
    pub fn get_database_backend(&self) -> DbBackend {
        self.proxy
            .lock()
            .expect("Fail to acquire mocker")
            .get_database_backend()
    }

    /// Execute the SQL statement in the [ProxyDatabase]
    #[instrument(level = "trace")]
    pub fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", statement);
        self.proxy.lock().map_err(exec_err)?.execute(statement)
    }

    /// Return one [QueryResult] if the query was successful
    #[instrument(level = "trace")]
    pub fn query_one(&self, statement: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let result = self.proxy.lock().map_err(query_err)?.query(statement)?;
        Ok(result.into_iter().next())
    }

    /// Return all [QueryResult]s if the query was successful
    #[instrument(level = "trace")]
    pub fn query_all(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        self.proxy.lock().map_err(query_err)?.query(statement)
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
