use crate::{
    error::*, DatabaseConnection, DbBackend, EntityTrait, ExecResult, Iden, IdenStatic, Iterable,
    ModelTrait, ProxyDatabaseConnection, ProxyDatabaseTrait, QueryResult, SelectA, SelectB,
    Statement,
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

#[cfg(feature = "proxy")]
/// Defines a Proxy database suitable for testing
#[derive(Debug, Clone)]
pub struct ProxyDatabase {
    db_backend: DbBackend,
    proxy_func: Arc<dyn ProxyDatabaseFuncTrait>,
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

/// A trait to get a [ProxyRow] from a type useful for testing in the [ProxyDatabase]
pub trait IntoProxyRow {
    /// The method to perform this operation
    fn into_mock_row(self) -> ProxyRow;
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
        self.proxy_func.execute(statement)
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

    /// An iterator over the keys and values of a mock row
    pub fn into_column_value_tuples(self) -> impl Iterator<Item = (String, Value)> {
        self.values.into_iter()
    }
}

impl IntoProxyRow for ProxyRow {
    fn into_mock_row(self) -> ProxyRow {
        self
    }
}

impl<M> IntoProxyRow for M
where
    M: ModelTrait,
{
    fn into_mock_row(self) -> ProxyRow {
        let mut values = BTreeMap::new();
        for col in <<M::Entity as EntityTrait>::Column>::iter() {
            values.insert(col.to_string(), self.get(col));
        }
        ProxyRow { values }
    }
}

impl<M, N> IntoProxyRow for (M, N)
where
    M: ModelTrait,
    N: ModelTrait,
{
    fn into_mock_row(self) -> ProxyRow {
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

impl<M, N> IntoProxyRow for (M, Option<N>)
where
    M: ModelTrait,
    N: ModelTrait,
{
    fn into_mock_row(self) -> ProxyRow {
        let mut mapped_join = BTreeMap::new();

        for column in <<M as ModelTrait>::Entity as EntityTrait>::Column::iter() {
            mapped_join.insert(
                format!("{}{}", SelectA.as_str(), column.as_str()),
                self.0.get(column),
            );
        }
        if let Some(b_entity) = self.1 {
            for column in <<N as ModelTrait>::Entity as EntityTrait>::Column::iter() {
                mapped_join.insert(
                    format!("{}{}", SelectB.as_str(), column.as_str()),
                    b_entity.get(column),
                );
            }
        }

        mapped_join.into_mock_row()
    }
}

impl<T> IntoProxyRow for BTreeMap<T, Value>
where
    T: Into<String>,
{
    fn into_mock_row(self) -> ProxyRow {
        ProxyRow {
            values: self.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }
}
