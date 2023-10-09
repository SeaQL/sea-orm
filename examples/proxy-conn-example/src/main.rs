//! Proxy connection example.

#![deny(missing_docs)]

use sea_orm::{
    Database, DbBackend, DbErr, ExecResult, ProxyDatabaseTrait, ProxyExecResult, ProxyQueryRow,
    Statement,
};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct ProxyDb {}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyQueryRow>, DbErr> {
        println!("SQL query: {}", statement.sql);
        Ok(vec![])
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        println!("SQL execute: {}", statement.sql);
        Ok(ProxyExecResult::default())
    }
}

#[async_std::main]
async fn main() {
    let db = Database::connect_proxy(DbBackend::MySql, Arc::new(Mutex::new(Box::new(ProxyDb {}))))
        .await
        .unwrap();

    println!("{:?}", db);
}
