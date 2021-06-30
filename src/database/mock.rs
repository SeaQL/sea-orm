use crate::{
    error::*, DatabaseConnection, EntityTrait, ExecResult, ExecResultHolder, Iden, Iterable,
    MockDatabaseConnection, MockDatabaseTrait, ModelTrait, QueryResult, QueryResultRow, Statement,
    Transaction,
};
use sea_query::{Value, ValueType};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MockDatabase {
    transaction_log: Vec<Transaction>,
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

pub trait IntoMockRow {
    fn into_mock_row(self) -> MockRow;
}

impl<M> IntoMockRow for M
where
    M: ModelTrait,
{
    fn into_mock_row(self) -> MockRow {
        let mut values = BTreeMap::new();
        for col in <<M::Entity as EntityTrait>::Column>::iter() {
            values.insert(col.to_string(), self.get(col));
        }
        MockRow { values }
    }
}

impl MockDatabase {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_connection(self) -> DatabaseConnection {
        DatabaseConnection::MockDatabaseConnection(MockDatabaseConnection::new(self))
    }

    pub fn append_exec_results(mut self, mut vec: Vec<MockExecResult>) -> Self {
        self.exec_results.append(&mut vec);
        self
    }

    pub fn append_query_results<T>(mut self, vec: Vec<Vec<T>>) -> Self
    where
        T: IntoMockRow,
    {
        for row in vec.into_iter() {
            let row = row.into_iter().map(|vec| vec.into_mock_row()).collect();
            self.query_results.push(row);
        }
        self
    }
}

impl MockDatabaseTrait for MockDatabase {
    fn execute(&mut self, counter: usize, statement: Statement) -> Result<ExecResult, SeaErr> {
        self.transaction_log.push(Transaction::one(statement));
        if counter < self.exec_results.len() {
            Ok(ExecResult {
                result: ExecResultHolder::Mock(std::mem::take(&mut self.exec_results[counter])),
            })
        } else {
            Err(SeaErr::Execution)
        }
    }

    fn query(
        &mut self,
        counter: usize,
        statement: Statement,
    ) -> Result<Vec<QueryResult>, SeaErr> {
        self.transaction_log.push(Transaction::one(statement));
        if counter < self.query_results.len() {
            Ok(std::mem::take(&mut self.query_results[counter])
                .into_iter()
                .map(|row| QueryResult {
                    row: QueryResultRow::Mock(row),
                })
                .collect())
        } else {
            Err(SeaErr::Query)
        }
    }

    fn drain_transaction_log(&mut self) -> Vec<Transaction> {
        std::mem::take(&mut self.transaction_log)
    }
}

impl MockRow {
    pub fn try_get<T>(&self, col: &str) -> Result<T, SeaErr>
    where
        T: ValueType,
    {
        Ok(self.values.get(col).unwrap().clone().unwrap())
    }

    pub fn into_column_value_tuples(self) -> impl Iterator<Item = (String, Value)> {
        self.values.into_iter()
    }
}

impl IntoMockRow for BTreeMap<&str, Value> {
    fn into_mock_row(self) -> MockRow {
        MockRow {
            values: self.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        }
    }
}
