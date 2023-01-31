use crate::{
    error::*, DatabaseConnection, DbBackend, EntityTrait, ExecResult, ExecResultHolder, Iden,
    IdenStatic, Iterable, MockDatabaseConnection, MockDatabaseTrait, ModelTrait, QueryResult,
    QueryResultRow, SelectA, SelectB, Statement,
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
    exec_results: Vec<Result<MockExecResult, DbErr>>,
    query_results: Vec<Result<Vec<MockRow>, DbErr>>,
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

    /// Add some [MockExecResult]s to `exec_results`
    pub fn append_exec_results<I>(mut self, vec: I) -> Self
    where
        I: IntoIterator<Item = MockExecResult>,
    {
        self.exec_results.extend(vec.into_iter().map(Result::Ok));
        self
    }

    /// Add some Values to `query_results`
    pub fn append_query_results<T, I, II>(mut self, vec: II) -> Self
    where
        T: IntoMockRow,
        I: IntoIterator<Item = T>,
        II: IntoIterator<Item = I>,
    {
        for row in vec.into_iter() {
            let row = row.into_iter().map(|vec| Ok(vec.into_mock_row())).collect();
            self.query_results.push(row);
        }
        self
    }

    /// Add some [DbErr]s to `exec_results`
    pub fn append_exec_errors<I>(mut self, vec: I) -> Self
    where
        I: IntoIterator<Item = DbErr>,
    {
        self.exec_results.extend(vec.into_iter().map(Result::Err));
        self
    }

    /// Add some [DbErr]s to `query_results`
    pub fn append_query_errors<I>(mut self, vec: I) -> Self
    where
        I: IntoIterator<Item = DbErr>,
    {
        self.query_results.extend(vec.into_iter().map(Result::Err));
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
            match std::mem::replace(
                &mut self.exec_results[counter],
                Err(DbErr::Exec(RuntimeErr::Internal(
                    "this value has been consumed already".to_owned(),
                ))),
            ) {
                Ok(result) => Ok(ExecResult {
                    result: ExecResultHolder::Mock(result),
                }),
                Err(err) => Err(err),
            }
        } else {
            Err(DbErr::Exec(RuntimeErr::Internal(
                "`exec_results` buffer is empty".to_owned(),
            )))
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
            match std::mem::replace(
                &mut self.query_results[counter],
                Err(DbErr::Query(RuntimeErr::Internal(
                    "this value has been consumed already".to_owned(),
                ))),
            ) {
                Ok(result) => Ok(result
                    .into_iter()
                    .map(|row| QueryResult {
                        row: QueryResultRow::Mock(row),
                    })
                    .collect()),
                Err(err) => Err(err),
            }
        } else {
            Err(query_err("`query_results` buffer is empty."))
        }
    }

    #[instrument(level = "trace")]
    fn begin(&mut self) -> Result<(), DbErr> {
        match self.transaction.as_mut() {
            Some(transaction) => transaction.begin_nested(self.db_backend),
            None => self.transaction = Some(OpenTransaction::init()),
        };
        Ok(())
    }

    #[instrument(level = "trace")]
    fn commit(&mut self) -> Result<(), DbErr> {
        match self.transaction.as_mut() {
            Some(transaction) => {
                if transaction.commit(self.db_backend) {
                    let transaction = self.transaction.take().ok_or(exec_err(
                        "There is no open transaction to commit".to_owned(),
                    ))?;
                    self.transaction_log.push(transaction.into_transaction()?);
                }
                Ok(())
            }
            None => Err(exec_err("There is no open transaction to commit")),
        }
    }

    #[instrument(level = "trace")]
    fn rollback(&mut self) -> Result<(), DbErr> {
        match self.transaction.as_mut() {
            Some(transaction) => {
                if transaction.rollback(self.db_backend) {
                    let transaction = self
                        .transaction
                        .take()
                        .ok_or(exec_err("There is no open transaction to commit"))?;
                    self.transaction_log.push(transaction.into_transaction()?);
                }
                Ok(())
            }
            None => Err(exec_err("There is no open transaction to rollback")),
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
    /// Get a value from the [MockRow]
    pub fn try_get<T, I: crate::ColIdx>(&self, index: I) -> Result<T, DbErr>
    where
        T: ValueType,
    {
        if let Some(index) = index.as_str() {
            T::try_from(
                self.values
                    .get(index)
                    .ok_or_else(|| {
                        DbErr::Query(RuntimeErr::Internal(format!(
                            "No column for ColIdx {index:?}"
                        )))
                    })?
                    .clone(),
            )
            .map_err(|e| DbErr::Type(e.to_string()))
        } else if let Some(index) = index.as_usize() {
            let (_, value) = self.values.iter().nth(*index).ok_or_else(|| {
                DbErr::Query(RuntimeErr::Internal(format!(
                    "Column at index {index} not found"
                )))
            })?;
            T::try_from(value.clone()).map_err(|e| DbErr::Type(e.to_string()))
        } else {
            unreachable!("Missing ColIdx implementation for MockRow");
        }
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
            mapped_join.insert(
                format!("{}{}", SelectA.as_str(), column.as_str()),
                self.0.get(column),
            );
        }
        for column in <<N as ModelTrait>::Entity as EntityTrait>::Column::iter() {
            mapped_join.insert(
                format!("{}{}", SelectB.as_str(), column.as_str()),
                self.1.get(column),
            );
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
        match self.transaction_depth {
            0 => {
                self.push(Statement::from_string(db_backend, "COMMIT".to_owned()));
                true
            }
            _ => {
                self.push(Statement::from_string(
                    db_backend,
                    format!("RELEASE SAVEPOINT savepoint_{}", self.transaction_depth),
                ));
                self.transaction_depth -= 1;
                false
            }
        }
    }

    fn rollback(&mut self, db_backend: DbBackend) -> bool {
        match self.transaction_depth {
            0 => {
                self.push(Statement::from_string(db_backend, "ROLLBACK".to_owned()));
                true
            }
            _ => {
                self.push(Statement::from_string(
                    db_backend,
                    format!("ROLLBACK TO SAVEPOINT savepoint_{}", self.transaction_depth),
                ));
                self.transaction_depth -= 1;
                false
            }
        }
    }

    fn push(&mut self, stmt: Statement) {
        self.stmts.push(stmt);
    }

    fn into_transaction(self) -> Result<Transaction, DbErr> {
        match self.transaction_depth {
            0 => Ok(Transaction { stmts: self.stmts }),
            _ => Err(exec_err("There is uncommitted nested transaction")),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use crate::{
        entity::*, tests_cfg::*, DbBackend, DbErr, IntoMockRow, MockDatabase, RuntimeErr,
        Statement, Transaction, TransactionError, TransactionTrait,
    };
    use pretty_assertions::assert_eq;

    #[derive(Debug, PartialEq, Eq)]
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
            [
                Transaction::many([
                    Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                    Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                        [1u64.into()]
                    ),
                    Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
                        []
                    ),
                    Statement::from_string(DbBackend::Postgres, "COMMIT".to_owned()),
                ]),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
                    []
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
            _ => assert!(false),
        }

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([
                Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                    [1u64.into()]
                ),
                Statement::from_string(DbBackend::Postgres, "ROLLBACK".to_owned()),
            ])]
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
            [Transaction::many([
                Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                    [1u64.into()]
                ),
                Statement::from_string(DbBackend::Postgres, "SAVEPOINT savepoint_1".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
                    []
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
            [Transaction::many([
                Statement::from_string(DbBackend::Postgres, "BEGIN".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
                    [1u64.into()]
                ),
                Statement::from_string(DbBackend::Postgres, "SAVEPOINT savepoint_1".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id" FROM "fruit""#,
                    []
                ),
                Statement::from_string(DbBackend::Postgres, "SAVEPOINT savepoint_2".to_owned()),
                Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
                    []
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
            .append_query_results([[apple.clone(), orange.clone()]])
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
            .append_query_results([Vec::<fruit::Model>::new()])
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
            .append_query_results([[apple.clone(), orange.clone()]])
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
        let row = (
            cake::Model {
                id: 1,
                name: "Apple Cake".to_owned(),
            },
            fruit::Model {
                id: 2,
                name: "Apple".to_owned(),
                cake_id: Some(1),
            },
        );
        let mocked_row = row.into_mock_row();

        let a_id = mocked_row.try_get::<i32, _>("A_id");
        assert!(a_id.is_ok());
        assert_eq!(1, a_id.unwrap());
        let b_id = mocked_row.try_get::<i32, _>("B_id");
        assert!(b_id.is_ok());
        assert_eq!(2, b_id.unwrap());
    }

    #[smol_potat::test]
    async fn test_find_also_related_1() -> Result<(), DbErr> {
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[(
                cake::Model {
                    id: 1,
                    name: "Apple Cake".to_owned(),
                },
                fruit::Model {
                    id: 2,
                    name: "Apple".to_owned(),
                    cake_id: Some(1),
                },
            )]])
            .into_connection();

        assert_eq!(
            cake::Entity::find()
                .find_also_related(fruit::Entity)
                .all(&db)
                .await?,
            [(
                cake::Model {
                    id: 1,
                    name: "Apple Cake".to_owned(),
                },
                Some(fruit::Model {
                    id: 2,
                    name: "Apple".to_owned(),
                    cake_id: Some(1),
                })
            )]
        );

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "cake"."id" AS "A_id", "cake"."name" AS "A_name", "fruit"."id" AS "B_id", "fruit"."name" AS "B_name", "fruit"."cake_id" AS "B_cake_id" FROM "cake" LEFT JOIN "fruit" ON "cake"."id" = "fruit"."cake_id""#,
                []
            ),]
        );

        Ok(())
    }

    #[cfg(feature = "postgres-array")]
    #[smol_potat::test]
    async fn test_postgres_array_1() -> Result<(), DbErr> {
        mod collection {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "collection")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                pub integers: Vec<i32>,
                pub integers_opt: Option<Vec<i32>>,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
                collection::Model {
                    id: 1,
                    integers: vec![1, 2, 3],
                    integers_opt: Some(vec![1, 2, 3]),
                },
                collection::Model {
                    id: 2,
                    integers: vec![],
                    integers_opt: Some(vec![]),
                },
                collection::Model {
                    id: 3,
                    integers: vec![3, 1, 4],
                    integers_opt: None,
                },
            ]])
            .into_connection();

        assert_eq!(
            collection::Entity::find().all(&db).await?,
            [
                collection::Model {
                    id: 1,
                    integers: vec![1, 2, 3],
                    integers_opt: Some(vec![1, 2, 3]),
                },
                collection::Model {
                    id: 2,
                    integers: vec![],
                    integers_opt: Some(vec![]),
                },
                collection::Model {
                    id: 3,
                    integers: vec![3, 1, 4],
                    integers_opt: None,
                },
            ]
        );

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT "collection"."id", "collection"."integers", "collection"."integers_opt" FROM "collection""#,
                []
            ),]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn test_query_err() {
        let db = MockDatabase::new(DbBackend::MySql)
            .append_query_errors([DbErr::Query(RuntimeErr::Internal(
                "this is a mock query error".to_owned(),
            ))])
            .into_connection();

        assert_eq!(
            cake::Entity::find().all(&db).await,
            Err(DbErr::Query(RuntimeErr::Internal(
                "this is a mock query error".to_owned()
            )))
        );
    }

    #[smol_potat::test]
    async fn test_exec_err() {
        let db = MockDatabase::new(DbBackend::MySql)
            .append_exec_errors([DbErr::Exec(RuntimeErr::Internal(
                "this is a mock exec error".to_owned(),
            ))])
            .into_connection();

        let model = cake::ActiveModel::new();

        assert_eq!(
            model.save(&db).await,
            Err(DbErr::Exec(RuntimeErr::Internal(
                "this is a mock exec error".to_owned()
            )))
        );
    }
}
