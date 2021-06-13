use crate::{ExecErr, ExecResult, MockDatabaseTrait, QueryErr, QueryResult, Statement, TypeErr};
use sea_query::{Value, ValueType};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MockDatabase {
    transaction_log: Vec<Statement>,
    exec_results: Vec<ExecResult>,
    query_results: Vec<Vec<QueryResult>>,
}

#[derive(Clone, Debug)]
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
}

impl MockDatabaseTrait for MockDatabase {
    fn execute(&mut self, counter: usize, statement: Statement) -> Result<ExecResult, ExecErr> {
        self.transaction_log.push(statement);
        if counter < self.exec_results.len() {
            Err(ExecErr)
            // Ok(self.exec_results[counter].clone())
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
            Err(QueryErr)
            // Ok(self.query_results[counter].clone())
        } else {
            Err(QueryErr)
        }
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
