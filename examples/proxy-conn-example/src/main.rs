//! Proxy connection example.

#![deny(missing_docs)]

use std::sync::Arc;

use sea_orm::{
    ConnectOptions, Database, DbBackend, DbErr, ExecResult, ProxyDatabaseFuncTrait,
    ProxyExecResult, QueryResult, Statement,
};

#[derive(Debug)]
struct ProxyDb {}

impl ProxyDatabaseFuncTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<QueryResult>, DbErr> {
        println!("SQL query: {}", statement.sql);
        Ok(vec![])
    }

    fn execute(&self, statement: Statement) -> Result<ExecResult, DbErr> {
        println!("SQL execute: {}", statement.sql);
        Ok(ProxyExecResult::default().into())
    }
}

#[async_std::main]
async fn main() {
    let mut option = ConnectOptions::new("");
    option.proxy_type(DbBackend::MySql);
    option.proxy_func(Arc::new(ProxyDb {}));
    let db = Database::connect(option).await.unwrap();

    println!("{:?}", db);
}
