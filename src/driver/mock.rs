use crate::{
    debug_print, ConnectionErr, DatabaseConnection, ExecErr, ExecResult, MockDatabase, QueryErr,
    QueryResult, Statement,
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
    fn execute(&mut self, counter: usize, stmt: Statement) -> Result<ExecResult, ExecErr>;

    fn query(&mut self, counter: usize, stmt: Statement) -> Result<Vec<QueryResult>, QueryErr>;
}

impl MockDatabaseConnector {
    pub fn accepts(_string: &str) -> bool {
        true
    }

    pub async fn connect(_string: &str) -> Result<DatabaseConnection, ConnectionErr> {
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
}

impl MockDatabaseConnection {
    pub async fn execute(&self, statement: Statement) -> Result<ExecResult, ExecErr> {
        debug_print!("{}", statement);
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker
            .lock()
            .unwrap()
            .execute(self.counter.load(Ordering::SeqCst), statement)
    }

    pub async fn query_one(&self, statement: Statement) -> Result<Option<QueryResult>, QueryErr> {
        debug_print!("{}", statement);
        self.counter.fetch_add(1, Ordering::SeqCst);
        let result = self
            .mocker
            .lock()
            .unwrap()
            .query(self.counter.load(Ordering::SeqCst), statement)?;
        Ok(result.into_iter().next())
    }

    pub async fn query_all(&self, statement: Statement) -> Result<Vec<QueryResult>, QueryErr> {
        debug_print!("{}", statement);
        self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker
            .lock()
            .unwrap()
            .query(self.counter.load(Ordering::SeqCst), statement)
    }
}
