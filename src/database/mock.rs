use crate::{
    error::*, DatabaseConnection, DbBackend, EntityTrait, ExecResult, ExecResultHolder, Iden,
    Iterable, MockDatabaseConnection, MockDatabaseTrait, ModelTrait, QueryResult, QueryResultRow,
    Statement,
};
use sea_query::{Value, ValueType, Values};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug)]
pub struct MockDatabase {
    db_backend: DbBackend,
    transaction: Option<OpenTransaction>,
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

#[derive(Debug)]
pub struct OpenTransaction {
    stmts: Vec<Statement>,
    transaction_depth: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    stmts: Vec<Statement>,
}

impl MockDatabase {
    pub fn new(db_backend: DbBackend) -> Self {
        Self {
            db_backend,
            transaction: None,
            transaction_log: Vec::new(),
            exec_results: Vec::new(),
            query_results: Vec::new(),
        }
    }

    pub fn into_connection(self) -> DatabaseConnection {
        DatabaseConnection::MockDatabaseConnection(Arc::new(MockDatabaseConnection::new(self)))
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
    fn execute(&mut self, counter: usize, statement: Statement) -> Result<ExecResult, DbErr> {
        if let Some(transaction) = &mut self.transaction {
            transaction.push(statement);
        } else {
            self.transaction_log.push(Transaction::one(statement));
        }
        if counter < self.exec_results.len() {
            Ok(ExecResult {
                result: ExecResultHolder::Mock(std::mem::take(&mut self.exec_results[counter])),
            })
        } else {
            Err(DbErr::Exec("`exec_results` buffer is empty.".to_owned()))
        }
    }

    fn query(&mut self, counter: usize, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        if let Some(transaction) = &mut self.transaction {
            transaction.push(statement);
        } else {
            self.transaction_log.push(Transaction::one(statement));
        }
        if counter < self.query_results.len() {
            Ok(std::mem::take(&mut self.query_results[counter])
                .into_iter()
                .map(|row| QueryResult {
                    row: QueryResultRow::Mock(row),
                })
                .collect())
        } else {
            Err(DbErr::Query("`query_results` buffer is empty.".to_owned()))
        }
    }

    fn begin(&mut self) {
        if self.transaction.is_some() {
            panic!("There is uncommitted transaction");
        } else {
            self.transaction = Some(OpenTransaction::init());
        }
    }

    fn commit(&mut self) {
        if self.transaction.is_some() {
            let transaction = self.transaction.take().unwrap();
            self.transaction_log
                .push(transaction.into_transaction());
        } else {
            panic!("There is no open transaction to commit");
        }
    }

    fn rollback(&mut self) {
        if self.transaction.is_some() {
            self.transaction = None;
        } else {
            panic!("There is no open transaction to rollback");
        }
    }

    fn drain_transaction_log(&mut self) -> Vec<Transaction> {
        std::mem::take(&mut self.transaction_log)
    }

    fn get_database_backend(&self) -> DbBackend {
        self.db_backend
    }
}

impl MockRow {
    pub fn try_get<T>(&self, col: &str) -> Result<T, DbErr>
    where
        T: ValueType,
    {
        T::try_from(self.values.get(col).unwrap().clone()).map_err(|e| DbErr::Query(e.to_string()))
    }

    pub fn into_column_value_tuples(self) -> impl Iterator<Item = (String, Value)> {
        self.values.into_iter()
    }
}

impl IntoMockRow for MockRow {
    fn into_mock_row(self) -> MockRow {
        self
    }
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

impl IntoMockRow for BTreeMap<&str, Value> {
    fn into_mock_row(self) -> MockRow {
        MockRow {
            values: self.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
        }
    }
}

impl Transaction {
    pub fn from_sql_and_values<I>(db_backend: DbBackend, sql: &str, values: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Self::one(Statement::from_string_values_tuple(
            db_backend,
            (sql.to_string(), Values(values.into_iter().collect())),
        ))
    }

    /// Create a Transaction with one statement
    pub fn one(stmt: Statement) -> Self {
        Self { stmts: vec![stmt] }
    }

    /// Create a Transaction with many statements
    pub fn many<I>(stmts: I) -> Self
    where
        I: IntoIterator<Item = Statement>,
    {
        Self {
            stmts: stmts.into_iter().collect(),
        }
    }

    /// Wrap each Statement as a single-statement Transaction
    pub fn wrap<I>(stmts: I) -> Vec<Self>
    where
        I: IntoIterator<Item = Statement>,
    {
        stmts.into_iter().map(Self::one).collect()
    }
}

impl OpenTransaction {
    fn init() -> Self {
        Self {
            stmts: Vec::new(),
            transaction_depth: 0,
        }
    }

    fn push(&mut self, stmt: Statement) {
        self.stmts.push(stmt);
    }

    fn into_transaction(self) -> Transaction {
        Transaction { stmts: self.stmts }
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use crate::{
        entity::*, tests_cfg::*, ConnectionTrait, DbBackend, DbErr, MockDatabase, Transaction,
        Statement,
    };

    #[smol_potat::test]
    async fn test_transaction_1() {
        let db = MockDatabase::new(DbBackend::Postgres).into_connection();

        db.transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
                let _1 = cake::Entity::find().one(txn).await;
                let _2 = fruit::Entity::find().all(txn).await;

                Ok(())
            })
        })
        .await
        .unwrap();

        let _ = cake::Entity::find().all(&db).await;

        assert_eq!(
            db.into_transaction_log(),
            vec![
                Transaction::many(vec![
                    Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                        vec![1u64.into()]
                    ),
                    Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
                        vec![]
                    ),
                ]),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
                    vec![]
                ),
            ]
        );
    }
}
