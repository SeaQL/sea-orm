use crate::{
    error::*, DatabaseConnection, DbBackend, EntityTrait, ExecResult, ExecResultHolder, Iden,
    IdenStatic, Iterable, MockDatabaseConnection, MockDatabaseTrait, ModelTrait, QueryResult,
    QueryResultRow, Statement, SelectA, SelectB
};
use sea_query::{Value, ValueType, Values};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;

/// Defines a Mock database suitable for testing
#[derive(Debug)]
pub struct MockDatabase {
    db_backend: DbBackend,
    transaction: Option<OpenTransaction>,
    transaction_log: Vec<Transaction>,
    exec_results: Vec<MockExecResult>,
    query_results: Vec<Vec<MockRow>>,
}

/// Defines the results obtained from a [MockDatabase]
#[derive(Clone, Debug, Default)]
pub struct MockExecResult {
    /// The last inserted id on auto-increment
    pub last_insert_id: u64,
    /// The number of rows affected by the database operation
    pub rows_affected: u64,
}

/// Defines the structure of a test Row for the [MockDatabase]
/// which is just a [BTreeMap]<[String], [Value]>
#[derive(Clone, Debug)]
pub struct MockRow {
    values: BTreeMap<String, Value>,
}

/// A trait to get a [MockRow] from a type useful for testing in the [MockDatabase]
pub trait IntoMockRow {
    /// The method to perform this operation
    fn into_mock_row(self) -> MockRow;
}

/// Defines a transaction that is has not been committed
#[derive(Debug)]
pub struct OpenTransaction {
    stmts: Vec<Statement>,
    transaction_depth: usize,
}

/// Defines a database transaction as it holds a Vec<[Statement]>
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    stmts: Vec<Statement>,
}

impl MockDatabase {
    /// Instantiate a mock database with a [DbBackend] to simulate real
    /// world SQL databases
    pub fn new(db_backend: DbBackend) -> Self {
        Self {
            db_backend,
            transaction: None,
            transaction_log: Vec::new(),
            exec_results: Vec::new(),
            query_results: Vec::new(),
        }
    }

    /// Create a database connection
    pub fn into_connection(self) -> DatabaseConnection {
        DatabaseConnection::MockDatabaseConnection(Arc::new(MockDatabaseConnection::new(self)))
    }

    /// Add the [MockExecResult]s to the `exec_results` field for `Self`
    pub fn append_exec_results(mut self, mut vec: Vec<MockExecResult>) -> Self {
        self.exec_results.append(&mut vec);
        self
    }

    /// Add the [MockExecResult]s to the `exec_results` field for `Self`
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
    #[instrument(level = "trace")]
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

    #[instrument(level = "trace")]
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

    #[instrument(level = "trace")]
    fn begin(&mut self) {
        if self.transaction.is_some() {
            self.transaction
                .as_mut()
                .unwrap()
                .begin_nested(self.db_backend);
        } else {
            self.transaction = Some(OpenTransaction::init());
        }
    }

    #[instrument(level = "trace")]
    fn commit(&mut self) {
        if self.transaction.is_some() {
            if self.transaction.as_mut().unwrap().commit(self.db_backend) {
                let transaction = self.transaction.take().unwrap();
                self.transaction_log.push(transaction.into_transaction());
            }
        } else {
            panic!("There is no open transaction to commit");
        }
    }

    #[instrument(level = "trace")]
    fn rollback(&mut self) {
        if self.transaction.is_some() {
            if self.transaction.as_mut().unwrap().rollback(self.db_backend) {
                let transaction = self.transaction.take().unwrap();
                self.transaction_log.push(transaction.into_transaction());
            }
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
    /// Try to get the values of a [MockRow] and fail gracefully on error
    pub fn try_get<T>(&self, col: &str) -> Result<T, DbErr>
    where
        T: ValueType,
    {
        T::try_from(self.values.get(col).unwrap().clone()).map_err(|e| DbErr::Query(e.to_string()))
    }

    /// An iterator over the keys and values of a mock row
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

impl<M, N> IntoMockRow for (M, N)
where
    M: ModelTrait,
    N: ModelTrait,
{
    fn into_mock_row(self) -> MockRow {
        let mut mapped_join = BTreeMap::new();

        for column in <<M as ModelTrait>::Entity as EntityTrait>::Column::iter() {
            mapped_join.insert(format!("{}{}", SelectA.as_str(), column.as_str()), self.0.get(column));
        }
        for column in <<N as ModelTrait>::Entity as EntityTrait>::Column::iter() {
            mapped_join.insert(format!("{}{}", SelectB.as_str(), column.as_str()), self.1.get(column));
        }

        mapped_join.into_mock_row()
    }
}

impl IntoMockRow for BTreeMap<String, Value> {
    fn into_mock_row(self) -> MockRow {
        MockRow {
            values: self.into_iter().map(|(k, v)| (k, v)).collect(),
        }
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
    /// Get the [Value]s from s raw SQL statement depending on the [DatabaseBackend](crate::DatabaseBackend)
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
            stmts: vec![Statement::from_string(
                DbBackend::Postgres,
                "BEGIN".to_owned(),
            )],
            transaction_depth: 0,
        }
    }

    fn begin_nested(&mut self, db_backend: DbBackend) {
        self.transaction_depth += 1;
        self.push(Statement::from_string(
            db_backend,
            format!("SAVEPOINT savepoint_{}", self.transaction_depth),
        ));
    }

    fn commit(&mut self, db_backend: DbBackend) -> bool {
        if self.transaction_depth == 0 {
            self.push(Statement::from_string(db_backend, "COMMIT".to_owned()));
            true
        } else {
            self.push(Statement::from_string(
                db_backend,
                format!("RELEASE SAVEPOINT savepoint_{}", self.transaction_depth),
            ));
            self.transaction_depth -= 1;
            false
        }
    }

    fn rollback(&mut self, db_backend: DbBackend) -> bool {
        if self.transaction_depth == 0 {
            self.push(Statement::from_string(db_backend, "ROLLBACK".to_owned()));
            true
        } else {
            self.push(Statement::from_string(
                db_backend,
                format!("ROLLBACK TO SAVEPOINT savepoint_{}", self.transaction_depth),
            ));
            self.transaction_depth -= 1;
            false
        }
    }

    fn push(&mut self, stmt: Statement) {
        self.stmts.push(stmt);
    }

    fn into_transaction(self) -> Transaction {
        if self.transaction_depth != 0 {
            panic!("There is uncommitted nested transaction.");
        }
        Transaction { stmts: self.stmts }
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use crate::{
        entity::*, tests_cfg::*, ConnectionTrait, DbBackend, DbErr, MockDatabase, Statement,
        Transaction, TransactionError, IntoMockRow,
    };
    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq)]
    pub struct MyErr(String);

    impl std::error::Error for MyErr {}

    impl std::fmt::Display for MyErr {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.0.as_str())
        }
    }

    #[smol_potat::test]
    async fn test_transaction_1() {
        let db = MockDatabase::new(DbBackend::Postgres).into_connection();

        db.transaction::<_, (), DbErr>(|txn| {
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
                    Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
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
                    Statement::from_string(DbBackend::Postgres, "COMMIT".to_owned()),
                ]),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
                    vec![]
                ),
            ]
        );
    }

    #[smol_potat::test]
    async fn test_transaction_2() {
        let db = MockDatabase::new(DbBackend::Postgres).into_connection();

        let result = db
            .transaction::<_, (), MyErr>(|txn| {
                Box::pin(async move {
                    let _ = cake::Entity::find().one(txn).await;
                    Err(MyErr("test".to_owned()))
                })
            })
            .await;

        match result {
            Err(TransactionError::Transaction(err)) => {
                assert_eq!(err, MyErr("test".to_owned()))
            }
            _ => panic!(),
        }

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![
                Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                    vec![1u64.into()]
                ),
                Statement::from_string(DbBackend::Postgres, "ROLLBACK".to_owned()),
            ]),]
        );
    }

    #[smol_potat::test]
    async fn test_nested_transaction_1() {
        let db = MockDatabase::new(DbBackend::Postgres).into_connection();

        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let _ = cake::Entity::find().one(txn).await;

                txn.transaction::<_, (), DbErr>(|txn| {
                    Box::pin(async move {
                        let _ = fruit::Entity::find().all(txn).await;

                        Ok(())
                    })
                })
                .await
                .unwrap();

                Ok(())
            })
        })
        .await
        .unwrap();

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![
                Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                    vec![1u64.into()]
                ),
                Statement::from_string(DbBackend::Postgres, "SAVEPOINT savepoint_1".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
                    vec![]
                ),
                Statement::from_string(
                    DbBackend::Postgres,
                    "RELEASE SAVEPOINT savepoint_1".to_owned()
                ),
                Statement::from_string(DbBackend::Postgres, "COMMIT".to_owned()),
            ]),]
        );
    }

    #[smol_potat::test]
    async fn test_nested_transaction_2() {
        let db = MockDatabase::new(DbBackend::Postgres).into_connection();

        db.transaction::<_, (), DbErr>(|txn| {
            Box::pin(async move {
                let _ = cake::Entity::find().one(txn).await;

                txn.transaction::<_, (), DbErr>(|txn| {
                    Box::pin(async move {
                        let _ = fruit::Entity::find().all(txn).await;

                        txn.transaction::<_, (), DbErr>(|txn| {
                            Box::pin(async move {
                                let _ = cake::Entity::find().all(txn).await;

                                Ok(())
                            })
                        })
                        .await
                        .unwrap();

                        Ok(())
                    })
                })
                .await
                .unwrap();

                Ok(())
            })
        })
        .await
        .unwrap();

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![
                Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                    vec![1u64.into()]
                ),
                Statement::from_string(DbBackend::Postgres, "SAVEPOINT savepoint_1".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
                    vec![]
                ),
                Statement::from_string(DbBackend::Postgres, "SAVEPOINT savepoint_2".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
                    vec![]
                ),
                Statement::from_string(
                    DbBackend::Postgres,
                    "RELEASE SAVEPOINT savepoint_2".to_owned()
                ),
                Statement::from_string(
                    DbBackend::Postgres,
                    "RELEASE SAVEPOINT savepoint_1".to_owned()
                ),
                Statement::from_string(DbBackend::Postgres, "COMMIT".to_owned()),
            ]),]
        );
    }

    #[smol_potat::test]
    async fn test_stream_1() -> Result<(), DbErr> {
        use futures::TryStreamExt;

        let apple = fruit::Model {
            id: 1,
            name: "Apple".to_owned(),
            cake_id: Some(1),
        };

        let orange = fruit::Model {
            id: 2,
            name: "orange".to_owned(),
            cake_id: None,
        };

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![apple.clone(), orange.clone()]])
            .into_connection();

        let mut stream = fruit::Entity::find().stream(&db).await?;

        assert_eq!(stream.try_next().await?, Some(apple));

        assert_eq!(stream.try_next().await?, Some(orange));

        assert_eq!(stream.try_next().await?, None);

        Ok(())
    }

    #[smol_potat::test]
    async fn test_stream_2() -> Result<(), DbErr> {
        use fruit::Entity as Fruit;
        use futures::TryStreamExt;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![Vec::<fruit::Model>::new()])
            .into_connection();

        let mut stream = Fruit::find().stream(&db).await?;

        while let Some(item) = stream.try_next().await? {
            let _item: fruit::ActiveModel = item.into();
        }

        Ok(())
    }

    #[smol_potat::test]
    async fn test_stream_in_transaction() -> Result<(), DbErr> {
        use futures::TryStreamExt;

        let apple = fruit::Model {
            id: 1,
            name: "Apple".to_owned(),
            cake_id: Some(1),
        };

        let orange = fruit::Model {
            id: 2,
            name: "orange".to_owned(),
            cake_id: None,
        };

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![apple.clone(), orange.clone()]])
            .into_connection();

        let txn = db.begin().await?;

        if let Ok(mut stream) = fruit::Entity::find().stream(&txn).await {
            assert_eq!(stream.try_next().await?, Some(apple));

            assert_eq!(stream.try_next().await?, Some(orange));

            assert_eq!(stream.try_next().await?, None);

            // stream will be dropped end of scope
        }

        txn.commit().await?;

        Ok(())
    }

    #[smol_potat::test]
    async fn test_mocked_join() {
        let mocked_row = (
            cake::Model {
                id: 1,
                name: "Apple Cake".to_owned(),
            },
            fruit::Model {
                id: 2,
                name: "Apple".to_owned(),
                cake_id: Some(1),
            }
        ).into_mock_row();

        let a_id = mocked_row.try_get::<i32>("A_id");
        assert!(a_id.is_ok());
        assert_eq!(1, a_id.unwrap());
        let b_id = mocked_row.try_get::<i32>("B_id");
        assert!(b_id.is_ok());
        assert_eq!(2, b_id.unwrap());
    }
}
