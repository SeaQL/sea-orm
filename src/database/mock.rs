use crate::{
    Database, DatabaseConnection, ExecErr, ExecResult, ExecResultHolder, MockDatabaseConnection,
    MockDatabaseTrait, QueryErr, QueryResult, QueryResultRow, Statement, TypeErr,
};
use sea_query::{Value, ValueType};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MockDatabase {
    transaction_log: Vec<Statement>,
    exec_results: Vec<MockExecResult>,
    query_results: Vec<Vec<MockRow>>,
}

#[derive(Clone, Debug, Default)]
pub struct MockExecResult {
    pub last_insert_id: u64,
    pub rows_affected: u64,
}

#[derive(Clone, Debug)]
pub struct MockRow {
    values: BTreeMap<String, Value>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_database(self) -> Database {
        Database {
            connection: DatabaseConnection::MockDatabaseConnection(MockDatabaseConnection::new(
                self,
            )),
        }
    }

    pub fn append_exec_results(mut self, mut vec: Vec<MockExecResult>) -> Self {
        self.exec_results.append(&mut vec);
        self
    }

    pub fn append_query_results(mut self, mut vec: Vec<Vec<MockRow>>) -> Self {
        self.query_results.append(&mut vec);
        self
    }

    pub fn into_transaction_log(self) -> Vec<Statement> {
        self.transaction_log
    }
}

impl MockDatabaseTrait for MockDatabase {
    fn execute(&mut self, counter: usize, statement: Statement) -> Result<ExecResult, ExecErr> {
        self.transaction_log.push(statement);
        if counter < self.exec_results.len() {
            Ok(ExecResult {
                result: ExecResultHolder::Mock(std::mem::take(&mut self.exec_results[counter])),
            })
        } else {
            Err(ExecErr)
        }
    }

    fn query(
        &mut self,
        counter: usize,
        statement: Statement,
    ) -> Result<Vec<QueryResult>, QueryErr> {
        self.transaction_log.push(statement);
        if counter < self.query_results.len() {
            Ok(std::mem::take(&mut self.query_results[counter])
                .into_iter()
                .map(|row| QueryResult {
                    row: QueryResultRow::Mock(row),
                })
                .collect())
        } else {
            Err(QueryErr)
        }
    }

    fn into_transaction_log(&mut self) -> Vec<Statement> {
        std::mem::take(&mut self.transaction_log)
    }
}

impl MockRow {
    pub fn try_get<T>(&self, col: &str) -> Result<T, TypeErr>
    where
        T: ValueType,
    {
        Ok(self.values.get(col).unwrap().clone().unwrap())
    }

    pub fn into_column_value_tuples(self) -> impl Iterator<Item = (String, Value)> {
        self.values.into_iter()
    }
}

impl From<BTreeMap<&str, Value>> for MockRow {
    fn from(values: BTreeMap<&str, Value>) -> Self {
        Self {
            values: values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        }
    }
}
