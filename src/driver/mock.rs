use crate::{
    debug_print, error::*, DatabaseConnection, DbBackend, ExecResult, MockDatabase, QueryResult,
    Statement, Transaction,
};
use std::{fmt::Debug, pin::Pin, sync::{Arc,
    atomic::{AtomicUsize, Ordering},
    Mutex,
}};
use futures::Stream;

#[derive(Debug)]
pub struct MockDatabaseConnector;

#[derive(Debug)]
pub struct MockDatabaseConnection {
    counter: AtomicUsize,
    mocker: Mutex<Box<dyn MockDatabaseTrait>>,
}

pub trait MockDatabaseTrait: Send + Debug {
    fn execute(&mut self, counter: usize, stmt: Statement) -> Result<ExecResult, DbErr>;

    fn query(&mut self, counter: usize, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    fn drain_transaction_log(&mut self) -> Vec<Transaction>;

    fn get_database_backend(&self) -> DbBackend;
}

impl MockDatabaseConnector {
    #[allow(unused_variables)]
    pub fn accepts(string: &str) -> bool {
        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return true;
        }
        #[cfg(feature = "sqlx-postgres")]
        if crate::SqlxPostgresConnector::accepts(string) {
            return true;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if crate::SqlxSqliteConnector::accepts(string) {
            return true;
        }
        false
    }

    #[allow(unused_variables)]
    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        macro_rules! connect_mock_db {
            ( $syntax: expr ) => {
                Ok(DatabaseConnection::MockDatabaseConnection(
                    Arc::new(MockDatabaseConnection::new(MockDatabase::new($syntax))),
                ))
            };
        }

        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return connect_mock_db!(DbBackend::MySql);
        }
        #[cfg(feature = "sqlx-postgres")]
        if crate::SqlxPostgresConnector::accepts(string) {
            return connect_mock_db!(DbBackend::Postgres);
        }
        #[cfg(feature = "sqlx-sqlite")]
        if crate::SqlxSqliteConnector::accepts(string) {
            return connect_mock_db!(DbBackend::Sqlite);
        }
        connect_mock_db!(DbBackend::Postgres)
    }
}

impl MockDatabaseConnection {
    pub fn new<M: 'static>(m: M) -> Self
    where
        M: MockDatabaseTrait,
    {
        Self {
            counter: AtomicUsize::new(0),
            mocker: Mutex::new(Box::new(m)),
        }
    }

    pub fn get_mocker_mutex(&self) -> &Mutex<Box<dyn MockDatabaseTrait>> {
        &self.mocker
    }

    pub fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker.lock().unwrap().execute(counter, statement)
    }

    pub fn query_one(&self, statement: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let result = self.mocker.lock().unwrap().query(counter, statement)?;
        Ok(result.into_iter().next())
    }

    pub fn query_all(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker.lock().unwrap().query(counter, statement)
    }

    pub fn fetch(&self, statement: &Statement) -> Pin<Box<dyn Stream<Item=Result<QueryResult, DbErr>>>> {
        match self.query_all(statement.clone()) {
            Ok(v) => Box::pin(futures::stream::iter(v.into_iter().map(|r| Ok(r)))),
            Err(e) => Box::pin(futures::stream::iter(Some(Err(e)).into_iter())),
        }
    }

    pub fn get_database_backend(&self) -> DbBackend {
        self.mocker.lock().unwrap().get_database_backend()
    }
}
