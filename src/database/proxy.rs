use crate::{
    error::*, DatabaseConnection, DbBackend, ExecResult, ExecResultHolder, ProxyDatabaseConnection,
    ProxyDatabaseTrait, QueryResult, Statement,
};

use sea_query::{Value, ValueType};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;

#[cfg(feature = "proxy")]
/// Defines the [ProxyDatabaseFuncTrait] to save the functions
pub trait ProxyDatabaseFuncTrait: Send + Sync + std::fmt::Debug {
    /// Execute a query in the [ProxyDatabase], and return the query results
    fn query(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr>;

    /// Execute a command in the [ProxyDatabase], and report the number of rows affected
    fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr>;

    /// Begin a transaction in the [ProxyDatabase]
    fn begin(&self);

    /// Commit a transaction in the [ProxyDatabase]
    fn commit(&self);

    /// Rollback a transaction in the [ProxyDatabase]
    fn rollback(&self);

    /// Ping the [ProxyDatabase], it should return an error if the database is not available
    fn ping(&self) -> Result<(), DbErr>;
}

/// Defines the results obtained from a [ProxyDatabase]
#[derive(Clone, Debug, Default)]
pub struct ProxyExecResult {
    /// The last inserted id on auto-increment
    pub last_insert_id: u64,
    /// The number of rows affected by the database operation
    pub rows_affected: u64,
}

/// Defines the structure of a test Row for the [ProxyDatabase]
/// which is just a [BTreeMap]<[String], [Value]>
#[derive(Clone, Debug)]
pub struct ProxyRow {
    values: BTreeMap<String, Value>,
}

#[cfg(feature = "proxy")]
/// Defines a Proxy database suitable for testing
#[derive(Debug, Clone)]
pub struct ProxyDatabase {
    db_backend: DbBackend,
    proxy_func: Arc<dyn ProxyDatabaseFuncTrait>,
}

impl ProxyDatabase {
    /// Instantiate a proxy database with a [DbBackend] and the [ProxyDatabaseFuncTrait]
    pub fn new(db_backend: DbBackend, func: Arc<dyn ProxyDatabaseFuncTrait>) -> Self {
        Self {
            db_backend,
            proxy_func: func.to_owned(),
        }
    }

    /// Create a database connection
    pub fn into_connection(self) -> DatabaseConnection {
        DatabaseConnection::ProxyDatabaseConnection(Arc::new(ProxyDatabaseConnection::new(self)))
    }
}

impl ProxyDatabaseTrait for ProxyDatabase {
    #[instrument(level = "trace")]
    fn execute(&mut self, statement: Statement) -> Result<ExecResult, DbErr> {
        match self.proxy_func.execute(statement) {
            Ok(result) => Ok(ExecResult {
                result: ExecResultHolder::Proxy(ProxyExecResult {
                    last_insert_id: result.last_insert_id(),
                    rows_affected: result.rows_affected(),
                }),
            }),
            Err(err) => Err(err),
        }
    }

    #[instrument(level = "trace")]
    fn query(&mut self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        self.proxy_func.query(statement)
    }

    #[instrument(level = "trace")]
    fn begin(&mut self) {
        self.proxy_func.begin()
    }

    #[instrument(level = "trace")]
    fn commit(&mut self) {
        self.proxy_func.commit()
    }

    #[instrument(level = "trace")]
    fn rollback(&mut self) {
        self.proxy_func.rollback()
    }

    fn get_database_backend(&self) -> DbBackend {
        self.db_backend
    }

    fn ping(&self) -> Result<(), DbErr> {
        self.proxy_func.ping()
    }
}

impl ProxyRow {
    /// Get a value from the [ProxyRow]
    pub fn try_get<T, I: crate::ColIdx>(&self, index: I) -> Result<T, DbErr>
    where
        T: ValueType,
    {
        if let Some(index) = index.as_str() {
            T::try_from(
                self.values
                    .get(index)
                    .ok_or_else(|| query_err(format!("No column for ColIdx {index:?}")))?
                    .clone(),
            )
            .map_err(type_err)
        } else if let Some(index) = index.as_usize() {
            let (_, value) = self
                .values
                .iter()
                .nth(*index)
                .ok_or_else(|| query_err(format!("Column at index {index} not found")))?;
            T::try_from(value.clone()).map_err(type_err)
        } else {
            unreachable!("Missing ColIdx implementation for ProxyRow");
        }
    }

    /// An iterator over the keys and values of a proxy row
    pub fn into_column_value_tuples(self) -> impl Iterator<Item = (String, Value)> {
        self.values.into_iter()
    }
}

#[cfg(test)]
#[cfg(feature = "proxy")]
mod tests {
    use crate::{
        entity::*, tests_cfg::*, DbBackend, DbErr, ExecResult, ProxyDatabase,
        ProxyDatabaseFuncTrait, QueryResult, Statement,
    };
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    #[derive(Debug)]
    struct EmptyProxyFunc {}

    impl ProxyDatabaseFuncTrait for EmptyProxyFunc {
        fn query(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
            println!("query: {:?}", statement);
            Ok(vec![])
        }

        fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr> {
            println!("execute: {:?}", statement);
            Ok(ExecResult {
                result: crate::ExecResultHolder::Proxy(crate::ProxyExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                }),
            })
        }

        fn begin(&self) {}

        fn commit(&self) {}

        fn rollback(&self) {}

        fn ping(&self) -> Result<(), DbErr> {
            Ok(())
        }
    }

    use once_cell::sync::Lazy;

    static EMPTY_DB_FUNC: Lazy<Arc<EmptyProxyFunc>> = Lazy::new(|| Arc::new(EmptyProxyFunc {}));

    #[smol_potat::test]
    async fn test_empty_oper() {
        let db =
            ProxyDatabase::new(DbBackend::MySql, (*EMPTY_DB_FUNC).to_owned()).into_connection();

        let _ = cake::Entity::find().all(&db).await;

        let item = cake::ActiveModel {
            id: NotSet,
            name: Set("Alice".to_string()),
        };
        cake::Entity::insert(item).exec(&db).await.unwrap();

        assert_eq!("1", "1");
    }
}
