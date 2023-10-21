use crate::{error::*, ExecResult, ExecResultHolder, QueryResult, QueryResultRow, Statement};

use sea_query::{Value, ValueType};
use std::{collections::BTreeMap, fmt::Debug};

#[cfg(feature = "proxy")]
/// Defines the [ProxyDatabaseTrait] to save the functions
pub trait ProxyDatabaseTrait: Send + Sync + std::fmt::Debug {
    /// Execute a query in the [ProxyDatabase], and return the query results
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr>;

    /// Execute a command in the [ProxyDatabase], and report the number of rows affected
    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr>;

    /// Begin a transaction in the [ProxyDatabase]
    fn begin(&self) {}

    /// Commit a transaction in the [ProxyDatabase]
    fn commit(&self) {}

    /// Rollback a transaction in the [ProxyDatabase]
    fn rollback(&self) {}

    /// Ping the [ProxyDatabase], it should return an error if the database is not available
    fn ping(&self) -> Result<(), DbErr> {
        Ok(())
    }
}

/// Defines the results obtained from a [ProxyDatabase]
#[cfg(feature = "proxy")]
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct ProxyExecResult {
    /// The last inserted id on auto-increment
    pub last_insert_id: u64,
    /// The number of rows affected by the database operation
    pub rows_affected: u64,
}

#[cfg(feature = "proxy")]
impl ProxyExecResult {
    /// Create a new [ProxyExecResult] from the last inserted id and the number of rows affected
    pub fn new(last_insert_id: u64, rows_affected: u64) -> Self {
        Self {
            last_insert_id,
            rows_affected,
        }
    }
}

#[cfg(feature = "proxy")]
impl Default for ExecResultHolder {
    fn default() -> Self {
        Self::Proxy(ProxyExecResult::default())
    }
}

#[cfg(feature = "proxy")]
impl From<ProxyExecResult> for ExecResult {
    fn from(result: ProxyExecResult) -> Self {
        Self {
            result: ExecResultHolder::Proxy(result),
        }
    }
}

#[cfg(feature = "proxy")]
impl From<ExecResult> for ProxyExecResult {
    fn from(result: ExecResult) -> Self {
        match result.result {
            #[cfg(feature = "sqlx-mysql")]
            ExecResultHolder::SqlxMySql(result) => Self {
                last_insert_id: result.last_insert_id() as u64,
                rows_affected: result.rows_affected(),
            },
            #[cfg(feature = "sqlx-postgres")]
            ExecResultHolder::SqlxPostgres(result) => Self {
                last_insert_id: result.last_insert_rowid() as u64,
                rows_affected: result.rows_affected(),
            },
            #[cfg(feature = "sqlx-sqlite")]
            ExecResultHolder::SqlxSqlite(result) => Self {
                last_insert_id: result.last_insert_rowid() as u64,
                rows_affected: result.rows_affected(),
            },
            #[cfg(feature = "mock")]
            ExecResultHolder::Mock(result) => Self {
                last_insert_id: result.last_insert_id,
                rows_affected: result.rows_affected,
            },
            ExecResultHolder::Proxy(result) => result,
        }
    }
}

/// Defines the structure of a Row for the [ProxyDatabase]
/// which is just a [BTreeMap]<[String], [Value]>
#[cfg(feature = "proxy")]
#[derive(Clone, Debug)]
pub struct ProxyRow {
    /// The values of the single row
    pub values: BTreeMap<String, Value>,
}

#[cfg(feature = "proxy")]
impl ProxyRow {
    /// Create a new [ProxyRow] from a [BTreeMap]<[String], [Value]>
    pub fn new(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }
}

#[cfg(feature = "proxy")]
impl Default for ProxyRow {
    fn default() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }
}

#[cfg(feature = "proxy")]
impl From<BTreeMap<String, Value>> for ProxyRow {
    fn from(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }
}

#[cfg(feature = "proxy")]
impl From<ProxyRow> for BTreeMap<String, Value> {
    fn from(row: ProxyRow) -> Self {
        row.values
    }
}

#[cfg(feature = "proxy")]
impl From<ProxyRow> for Vec<(String, Value)> {
    fn from(row: ProxyRow) -> Self {
        row.values.into_iter().collect()
    }
}

#[cfg(feature = "proxy")]
impl From<ProxyRow> for QueryResult {
    fn from(row: ProxyRow) -> Self {
        QueryResult {
            row: QueryResultRow::Proxy(row),
        }
    }
}

#[cfg(feature = "proxy")]
impl From<QueryResult> for ProxyRow {
    fn from(result: QueryResult) -> Self {
        match result.row {
            #[cfg(feature = "sqlx-mysql")]
            QueryResultRow::SqlxMySql(row) => row.into(),
            #[cfg(feature = "sqlx-postgres")]
            QueryResultRow::SqlxPostgres(row) => row.into(),
            #[cfg(feature = "sqlx-sqlite")]
            QueryResultRow::SqlxSqlite(row) => row.into(),
            #[cfg(feature = "mock")]
            QueryResultRow::Mock(row) => row.into(),
            QueryResultRow::Proxy(row) => row,
        }
    }
}

// #[cfg(all(feature = "proxy", feature = "sqlx-mysql"))]
// impl From<sqlx::mysql::MySqlRow> for ProxyRow {
//     fn from(row: sqlx::mysql::MySqlRow) -> Self {
//         use sqlx::{Column, Row, TypeInfo};
//         Self {
//             values: row
//                 .columns()
//                 .iter()
//                 .map(|c| {
//                     (
//                         c.name().to_string(),
//                         match c.type_info().name() {
//                             "MEDIUMINT" | "INT" | "INTEGER" => Value::Int(Some(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             )),
//                             "BIGINT" => Value::BigInt(Some(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             )),
//                             "FLOAT" => Value::Float(Some(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             )),
//                             "DOUBLE" => Value::Double(Some(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             )),

//                             #[cfg(feature = "with-bigdecimal")]
//                             "DECIMAL" => Value::BigDecimal(Some(Box::new(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             ))),

//                             #[cfg(feature = "with-chrono")]
//                             "DATE" => Value::ChronoDate(Some(Box::new(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             ))),
//                             #[cfg(feature = "with-chrono")]
//                             "TIME" => Value::ChronoTime(Some(Box::new(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             ))),
//                             #[cfg(feature = "with-chrono")]
//                             "DATETIME" => Value::ChronoDateTime(Some(Box::new(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             ))),

//                             "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT"
//                             | "LONGTEXT" => Value::String(Some(Box::new(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             ))),
//                             _ => Value::String(Some(Box::new(
//                                 row.try_get(c.ordinal())
//                                     .expect("Failed to get value from row"),
//                             ))),
//                         },
//                     )
//                 })
//                 .collect(),
//         }
//     }
// }

#[cfg(all(feature = "proxy", feature = "mock"))]
impl From<crate::MockRow> for ProxyRow {
    fn from(row: crate::MockRow) -> Self {
        Self {
            values: row.values.into_iter().collect(),
        }
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
        entity::*, tests_cfg::*, Database, DbBackend, DbErr, ProxyDatabaseTrait, ProxyExecResult,
        ProxyRow, Statement,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct ProxyDb {}

    impl ProxyDatabaseTrait for ProxyDb {
        fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
            println!("SQL query: {}", statement.sql);
            Ok(vec![].into())
        }

        fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
            println!("SQL execute: {}", statement.sql);
            Ok(ProxyExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            })
        }
    }

    #[smol_potat::test]
    async fn create_proxy_conn() {
        let db =
            Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
                .await
                .unwrap();
    }

    #[smol_potat::test]
    async fn select_rows() {
        let db =
            Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
                .await
                .unwrap();

        let _ = cake::Entity::find().all(&db).await;
    }

    #[smol_potat::test]
    async fn insert_one_row() {
        let db =
            Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
                .await
                .unwrap();

        let item = cake::ActiveModel {
            id: NotSet,
            name: Set("Alice".to_string()),
        };

        cake::Entity::insert(item).exec(&db).await.unwrap();
    }
}
