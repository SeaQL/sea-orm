use crate::{
    debug_print, error::*, DatabaseConnection, ExecResult, MockDatabase, QueryBuilderBackend,
    QueryResult, SchemaBuilderBackend, Statement, Syntax, Transaction,
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
    fn execute(&mut self, counter: usize, stmt: Statement) -> Result<ExecResult, DbErr>;

    fn query(&mut self, counter: usize, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    fn drain_transaction_log(&mut self) -> Vec<Transaction>;

    fn get_syntax(&self) -> Syntax;
}

impl MockDatabaseConnector {
    pub fn accepts(string: &str) -> bool {
        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return true;
        }
        #[cfg(feature = "sqlx-sqlite")]
        if crate::SqlxSqliteConnector::accepts(string) {
            return true;
        }
        false
    }

    pub async fn connect(string: &str) -> Result<DatabaseConnection, DbErr> {
        macro_rules! connect_mock_db {
            ( $syntax: expr ) => {
                Ok(DatabaseConnection::MockDatabaseConnection(
                    MockDatabaseConnection::new(MockDatabase::new($syntax)),
                ))
            };
        }

        #[cfg(feature = "sqlx-mysql")]
        if crate::SqlxMySqlConnector::accepts(string) {
            return connect_mock_db!(Syntax::MySql);
        }
        #[cfg(feature = "sqlx-sqlite")]
        if crate::SqlxSqliteConnector::accepts(string) {
            return connect_mock_db!(Syntax::Sqlite);
        }
        connect_mock_db!(Syntax::Postgres)
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

    pub async fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker.lock().unwrap().execute(counter, statement)
    }

    pub async fn query_one(&self, statement: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        let result = self.mocker.lock().unwrap().query(counter, statement)?;
        Ok(result.into_iter().next())
    }

    pub async fn query_all(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", statement);
        let counter = self.counter.fetch_add(1, Ordering::SeqCst);
        self.mocker.lock().unwrap().query(counter, statement)
    }

    pub fn get_query_builder_backend(&self) -> QueryBuilderBackend {
        self.mocker
            .lock()
            .unwrap()
            .get_syntax()
            .get_query_builder_backend()
    }

    pub fn get_schema_builder_backend(&self) -> SchemaBuilderBackend {
        self.mocker
            .lock()
            .unwrap()
            .get_syntax()
            .get_schema_builder_backend()
    }
}
