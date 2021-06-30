use crate::{
    debug_print, error::*, DatabaseConnection, ExecResult, MockDatabase, QueryResult, Statement,
    Transaction,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

pub struct MockDatabaseConnector;

pub struct MockDatabaseConnection {
    counter: AtomicUsize,
    mocker: Mutex<Box<dyn MockDatabaseTrait>>,
}

pub trait MockDatabaseTrait: Send {
    fn execute(&mut self, counter: usize, stmt: Statement) -> Result<ExecResult, SeaErr>;

    fn query(&mut self, counter: usize, stmt: Statement) -> Result<Vec<QueryResult>, SeaErr>;

    fn drain_transaction_log(&mut self) -> Vec<Transaction>;
}

impl MockDatabaseConnector {
    pub fn accepts(string: &str) -> bool {
        string.starts_with("mock://")
    }

    pub async fn connect(_string: &str) -> Result<DatabaseConnection, SeaErr> {
        Ok(DatabaseConnection::MockDatabaseConnection(
            MockDatabaseConnection::new(MockDatabase::new()),
        ))
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

    pub async fn execute(&self, statement: Statement) -> Result<ExecResult, SeaErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker.lock().unwrap().execute(counter, statement)
    }

    pub async fn query_one(&self, statement: Statement) -> Result<Option<QueryResult>, SeaErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let result = self.mocker.lock().unwrap().query(counter, statement)?;
        Ok(result.into_iter().next())
    }

    pub async fn query_all(&self, statement: Statement) -> Result<Vec<QueryResult>, SeaErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker.lock().unwrap().query(counter, statement)
    }
}
