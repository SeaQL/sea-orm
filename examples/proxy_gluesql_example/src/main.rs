//! Proxy connection example.

#![deny(missing_docs)]

mod entity;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use gluesql::prelude::{Glue, MemoryStorage};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait,
    ProxyDatabaseTrait, ProxyExecResult, ProxyRow, Statement,
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

#[async_trait::async_trait]
impl ProxyDatabaseTrait for ProxyDb {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let sql = if let Some(values) = statement.values {
            // Replace all the '?' with the statement values

            statement
                .sql
                .split("?")
                .collect::<Vec<&str>>()
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, item)| {
                    acc.push_str(item);
                    if i < values.0.len() {
                        acc.push_str(&format!("{}", values.0[i]));
                    }
                    acc
                })
        } else {
            statement.sql
        };
        println!("SQL query: {}", sql);

        let mut ret: Vec<ProxyRow> = vec![];
        async_std::task::block_on(async {
            let raw = self.mem.lock().unwrap().execute(sql).await.unwrap();
            for payload in raw.iter() {
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
                                        gluesql::prelude::Value::Uuid(val) => sea_orm::Value::Uuid(
                                            Some(Box::new(uuid::Uuid::from_u128(*val))),
                                        ),
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
        });

        Ok(ret)
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let sql = if let Some(values) = statement.values {
            // Replace all the '?' with the statement values

            statement
                .sql
                .split("?")
                .collect::<Vec<&str>>()
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (i, item)| {
                    acc.push_str(item);
                    if i < values.0.len() {
                        acc.push_str(&format!("{}", values.0[i]));
                    }
                    acc
                })
        } else {
            statement.sql
        };

        println!("SQL execute: {}", sql);
        async_std::task::block_on(async {
            self.mem.lock().unwrap().execute(sql).await.unwrap();
        });

        Ok(ProxyExecResult {
            last_insert_id: None,
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
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                text TEXT NOT NULL
            )
        "#,
    )
    .await
    .unwrap();

    let db = Database::connect_proxy(
        DbBackend::Sqlite,
        Arc::new(Box::new(ProxyDb {
            mem: Mutex::new(glue),
        })),
    )
    .await
    .unwrap();

    println!("Initialized");

    let data = ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        title: Set("Homo".to_owned()),
        text: Set("いいよ、来いよ".to_owned()),
    };
    data.insert(&db).await.unwrap();

    let data = ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        title: Set("Homo".to_owned()),
        text: Set("そうだよ".to_owned()),
    };
    Entity::insert(data).exec(&db).await.unwrap();

    let data = ActiveModel {
        id: Set("野兽邸".to_string()),
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
