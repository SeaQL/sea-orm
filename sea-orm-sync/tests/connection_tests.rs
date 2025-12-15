#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::prelude::*;

#[sea_orm_macros::test]
pub fn connection_ping() {
    let ctx = TestContext::new("connection_ping");

    ctx.db.ping().unwrap();

    ctx.delete();
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-mysql")]
pub fn connection_ping_closed_mysql() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed")));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_mysql_connection_pool().close();
    assert_eq!(
        ctx_ping.db.ping(),
        Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
    );

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(format!("{base_url}/connection_ping_closed"));
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(2));

    let db = sea_orm::Database::connect(opt).unwrap();

    fn transaction_blocked(db: &DatabaseConnection) {
        let _txn = sea_orm::TransactionTrait::begin(db).unwrap();
        // Occupy the only connection, thus forcing others fail to acquire connection
        tokio::time::sleep(std::time::Duration::from_secs(3));
    }

    fn transaction(db: &DatabaseConnection) {
        // Should fail to acquire
        let txn = sea_orm::TransactionTrait::begin(db);
        assert_eq!(
            txn.expect_err("should be a time out"),
            crate::DbErr::ConnectionAcquire(ConnAcquireErr::Timeout)
        )
    }

    tokio::join!(transaction_blocked(&db), transaction(&db));

    ctx.delete();
}

#[sea_orm_macros::test]
#[cfg(all(feature = "sqlx-sqlite", not(feature = "sync")))]
pub fn connection_ping_closed_sqlite() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed")));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_sqlite_connection_pool().close();
    assert_eq!(
        ctx_ping.db.ping(),
        Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
    );

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(base_url);
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(2));

    let db = sea_orm::Database::connect(opt).unwrap();

    fn transaction_blocked(db: &DatabaseConnection) {
        let _txn = sea_orm::TransactionTrait::begin(db).unwrap();
        // Occupy the only connection, thus forcing others fail to acquire connection
        tokio::time::sleep(std::time::Duration::from_secs(3));
    }

    fn transaction(db: &DatabaseConnection) {
        // Should fail to acquire
        let txn = sea_orm::TransactionTrait::begin(db);
        assert_eq!(
            txn.expect_err("should be a time out"),
            crate::DbErr::ConnectionAcquire(ConnAcquireErr::Timeout)
        )
    }

    tokio::join!(transaction_blocked(&db), transaction(&db));

    ctx.delete();
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
pub fn connection_ping_closed_postgres() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed")));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_postgres_connection_pool().close();
    assert_eq!(
        ctx_ping.db.ping(),
        Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
    );

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(format!("{base_url}/connection_ping_closed"));
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(2));

    let db = sea_orm::Database::connect(opt).unwrap();

    fn transaction_blocked(db: &DatabaseConnection) {
        let _txn = sea_orm::TransactionTrait::begin(db).unwrap();
        // Occupy the only connection, thus forcing others fail to acquire connection
        tokio::time::sleep(std::time::Duration::from_secs(3));
    }

    fn transaction(db: &DatabaseConnection) {
        // Should fail to acquire
        let txn = sea_orm::TransactionTrait::begin(db);
        assert_eq!(
            txn.expect_err("should be a time out"),
            crate::DbErr::ConnectionAcquire(ConnAcquireErr::Timeout)
        )
    }

    tokio::join!(transaction_blocked(&db), transaction(&db));

    ctx.delete();
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
pub fn connection_with_search_path_postgres() {
    let ctx = TestContext::new("connection_with_search_path");

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(format!("{base_url}/connection_with_search_path"));
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(2))
        .set_schema_search_path("schema-with-special-characters");

    let db = sea_orm::Database::connect(opt);
    assert!(db.is_ok());

    ctx.delete();
}
