//! Proxy connection example.

#![deny(missing_docs)]

mod entity;

use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use gluesql::{memory_storage::MemoryStorage, prelude::Glue};
use sea_orm::{
    ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait, ProxyDatabaseTrait, ProxyExecResult,
    ProxyInsertResult, ProxyRow, Statement,
};

use entity::post::{ActiveModel, Entity};

struct ProxyDb {
    mem: Mutex<Glue<MemoryStorage>>,
}

impl std::fmt::Debug for ProxyDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyDb").finish()
    }
}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        println!("SQL query: {:?}", statement);
        let sql = statement.sql.clone();

        let mut ret: Vec<ProxyRow> = vec![];
        for payload in self.mem.lock().unwrap().execute(sql).unwrap().iter() {
            match payload {
                gluesql::prelude::Payload::Select { labels, rows } => {
                    for row in rows.iter() {
                        let mut map = BTreeMap::new();
                        for (label, column) in labels.iter().zip(row.iter()) {
                            map.insert(
                                label.to_owned(),
                                match column {
                                    gluesql::prelude::Value::I64(val) => {
                                        sea_orm::Value::BigInt(Some(*val))
                                    }
                                    gluesql::prelude::Value::Str(val) => {
                                        sea_orm::Value::String(Some(Box::new(val.to_owned())))
                                    }
                                    _ => unreachable!("Unsupported value: {:?}", column),
                                },
                            );
                        }
                        ret.push(map.into());
                    }
                }
                _ => unreachable!("Unsupported payload: {:?}", payload),
            }
        }

        Ok(ret)
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        if let Some(values) = statement.values {
            // Replace all the '?' with the statement values
            let mut new_sql = statement.sql.clone();
            let mark_count = new_sql.matches('?').count();
            for (i, v) in values.0.iter().enumerate() {
                if i >= mark_count {
                    break;
                }
                new_sql = new_sql.replacen('?', &v.to_string(), 1);
            }
            println!("SQL execute: {}", new_sql);

            self.mem.lock().unwrap().execute(new_sql).unwrap();
        } else {
            self.mem.lock().unwrap().execute(statement.sql).unwrap();
        }

        Ok(ProxyExecResult {
            last_insert_id: ProxyInsertResult::Inserted(vec![json!(1)]),
            rows_affected: 1,
        })
    }
}

#[async_std::main]
async fn main() {
    let mem = MemoryStorage::default();
    let mut glue = Glue::new(mem);

    glue.execute(
        r#"
            CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                text TEXT NOT NULL
            )
        "#,
    )
    .unwrap();

    let db = Database::connect_proxy(
        DbBackend::Sqlite,
        Arc::new(Mutex::new(Box::new(ProxyDb {
            mem: Mutex::new(glue),
        }))),
    )
    .await
    .unwrap();

    println!("Initialized");

    let data = ActiveModel {
        id: Set(11),
        title: Set("Homo".to_owned()),
        text: Set("いいよ、来いよ".to_owned()),
    };
    Entity::insert(data).exec(&db).await.unwrap();
    let data = ActiveModel {
        id: Set(45),
        title: Set("Homo".to_owned()),
        text: Set("そうだよ".to_owned()),
    };
    Entity::insert(data).exec(&db).await.unwrap();
    let data = ActiveModel {
        id: Set(14),
        title: Set("Homo".to_owned()),
        text: Set("悔い改めて".to_owned()),
    };
    Entity::insert(data).exec(&db).await.unwrap();

    let list = Entity::find().all(&db).await.unwrap().to_vec();
    println!("Result: {:?}", list);
}

#[cfg(test)]
mod tests {
    #[smol_potat::test]
    async fn try_run() {
        crate::main()
    }
}
