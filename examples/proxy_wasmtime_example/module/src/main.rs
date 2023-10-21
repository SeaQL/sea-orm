//! Proxy connection example.

#![deny(missing_docs)]

mod entity;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use sea_orm::{
    ActiveValue::Set, Database, DbBackend, DbErr, EntityTrait, ProxyDatabaseTrait, ProxyExecResult,
    ProxyRow, Statement,
};

use entity::post::{ActiveModel, Entity};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum RequestMsg {
    Query(String),
    Execute(String),

    Debug(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum ResponseMsg {
    Query(Vec<serde_json::Value>),
    Execute(ProxyExecResult),

    None,
}

#[derive(Debug)]
struct ProxyDb {}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        let sql = statement.sql.clone();

        Ok(vec![])
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        let sql = {
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

                new_sql
            } else {
                statement.sql
            }
        };

        // Send the query to stdout
        let msg = RequestMsg::Execute(sql);
        let msg = serde_json::to_string(&msg).unwrap();
        println!("{}", msg);

        // Get the result from stdin
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let ret: ResponseMsg = serde_json::from_str(&input).unwrap();
        let ret = match ret {
            ResponseMsg::Execute(v) => v,
            _ => unreachable!(),
        };

        Ok(ret)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let db = Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
        .await
        .unwrap();

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
    println!(
        "{}",
        serde_json::to_string(&RequestMsg::Debug(format!("{:?}", list))).unwrap()
    );
}
