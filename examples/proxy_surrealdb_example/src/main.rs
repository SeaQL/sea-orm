//! Proxy connection example.

#![deny(missing_docs)]

mod entity;

use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use sea_orm::{
    ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait, ProxyDatabaseTrait, ProxyExecResult,
    ProxyInsertResult, ProxyRow, Statement,
};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};

use entity::post::{ActiveModel, Entity};

#[derive(Debug)]
struct ProxyDb {
    mem: Surreal<Db>,
}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        println!("SQL query: {:?}", statement);
        let sql = statement.sql.clone();
        let mut ret = async_std::task::block_on(async {
            // Surrealdb's grammar is not compatible with sea-orm's
            // so we need to remove the extra clauses
            // from "SELECT `from`.`col` FROM `from` WHERE `from`.`col` = xx"
            // to "SELECT `col` FROM `from` WHERE `col` = xx"

            // Get the first index of "FROM"
            let from_index = sql.find("FROM").unwrap();
            // Get the name after "FROM"
            let from_name = sql[from_index + 5..].split(' ').next().unwrap();
            // Delete the name before all the columns
            let new_sql = sql.replace(&format!("{}.", from_name), "");

            self.mem.query(new_sql).await
        })
        .unwrap();

        // Convert the result to sea-orm's format
        let ret: Vec<serde_json::Value> = ret.take(0).unwrap();
        println!("SQL query result: {}", serde_json::to_string(&ret).unwrap());
        let ret = ret
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (k, v) in row.as_object().unwrap().iter() {
                    if k == "id" {
                        // Get `tb` and `id` columns from surrealdb
                        // and convert them to sea-orm's `id`
                        let tb = v.as_object().unwrap().get("tb").unwrap().to_string();
                        let id = v
                            .as_object()
                            .unwrap()
                            .get("id")
                            .unwrap()
                            .get("String")
                            .unwrap();

                        // Remove the quotes
                        let tb = tb.to_string().replace("\"", "");
                        let id = id.to_string().replace("\"", "");

                        map.insert("id".to_owned(), format!("{}:{}", tb, id).into());
                        continue;
                    }

                    map.insert(k.to_owned(), v.to_owned());
                }
                serde_json::Value::Object(map)
            })
            .map(|v: serde_json::Value| {
                let mut ret: BTreeMap<String, sea_orm::Value> = BTreeMap::new();
                for (k, v) in v.as_object().unwrap().iter() {
                    ret.insert(
                        k.to_owned(),
                        match v {
                            serde_json::Value::Bool(b) => {
                                sea_orm::Value::TinyInt(if *b { Some(1) } else { Some(0) })
                            }
                            serde_json::Value::Number(n) => {
                                if n.is_i64() {
                                    sea_orm::Value::BigInt(Some(n.as_i64().unwrap()))
                                } else if n.is_u64() {
                                    sea_orm::Value::BigUnsigned(Some(n.as_u64().unwrap()))
                                } else if n.is_f64() {
                                    sea_orm::Value::Double(Some(n.as_f64().unwrap()))
                                } else {
                                    unreachable!()
                                }
                            }
                            serde_json::Value::String(s) => {
                                sea_orm::Value::String(Some(Box::new(s.to_owned())))
                            }
                            _ => sea_orm::Value::Json(Some(Box::new(v.to_owned()))),
                        },
                    );
                }
                ProxyRow { values: ret }
            })
            .collect::<Vec<_>>();

        Ok(ret)
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        async_std::task::block_on(async {
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

                self.mem.query(new_sql).await
            } else {
                self.mem.query(statement.sql).await
            }
        })
        .unwrap();

        Ok(ProxyExecResult {
            last_insert_id: ProxyInsertResult::Inserted(vec![json!(1)]),
            rows_affected: 1,
        })
    }
}

#[async_std::main]
async fn main() {
    let mem = Surreal::new::<Mem>(()).await.unwrap();
    mem.use_ns("test").use_db("post").await.unwrap();

    let db = Database::connect_proxy(
        DbBackend::MySql,
        Arc::new(Mutex::new(Box::new(ProxyDb { mem }))),
    )
    .await
    .unwrap();

    println!("Initialized");

    let data = ActiveModel {
        title: Set("Homo".to_owned()),
        text: Set("いいよ、来いよ".to_owned()),
        ..Default::default()
    };
    Entity::insert(data).exec(&db).await.unwrap();
    let data = ActiveModel {
        title: Set("Homo".to_owned()),
        text: Set("そうだよ".to_owned()),
        ..Default::default()
    };
    Entity::insert(data).exec(&db).await.unwrap();
    let data = ActiveModel {
        title: Set("Homo".to_owned()),
        text: Set("悔い改めて".to_owned()),
        ..Default::default()
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
