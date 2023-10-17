//! Proxy connection example.

#![deny(missing_docs)]

mod entity;

use std::sync::{Arc, Mutex};

use sea_orm::{
    ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait, ProxyDatabaseTrait, ProxyExecResult,
    ProxyRow, Statement,
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
        let ret = async_std::task::block_on(async { self.mem.query(sql).await }).unwrap();
        println!("SQL query result: {:?}", ret);
        Ok(vec![])
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
            last_insert_id: 1,
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
