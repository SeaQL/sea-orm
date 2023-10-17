use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use sea_orm::{
    Database, DbBackend, DbErr, ProxyDatabaseTrait, ProxyExecResult, ProxyRow, Statement,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub id: u32,
    pub data: String,
}

#[derive(Debug)]
struct ProxyDb {}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        Ok(vec![])
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        Ok(ProxyExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        })
    }
}

#[async_std::main]
async fn main() {
    if let Ok(db) =
        Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {})))).await
    {
        println!("Initialized {:?}", db);
    } else {
        println!("Failed to initialize");
    }
}
