pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::prelude::*;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn connection_ping() {
    let ctx = TestContext::new("connection_ping").await;

    ctx.db.ping().await.unwrap();

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-mysql")]
pub async fn connection_ping_closed_mysql() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed").await));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_mysql_connection_pool().close().await;
    assert_eq!(
        ctx_ping.db.ping().await,
        Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
    );

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(format!("{base_url}/connection_ping_closed"));
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(10));

    let db = sea_orm::Database::connect(opt).await.unwrap();

    async fn transaction_blocked(db: &DatabaseConnection) {
        let _txn = sea_orm::TransactionTrait::begin(db).await.unwrap();
        // Occupy the only connection, thus forcing others fail to acquire connection
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }

    async fn transaction(db: &DatabaseConnection) {
        // Should fail to acquire
        let txn = sea_orm::TransactionTrait::begin(db).await;
        assert_eq!(
            txn.expect_err("should be a time out"),
            crate::DbErr::ConnectionAcquire(ConnAcquireErr::Timeout)
        )
    }

    tokio::join!(transaction_blocked(&db), transaction(&db));

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-sqlite")]
pub async fn connection_ping_closed_sqlite() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed").await));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_sqlite_connection_pool().close().await;
    assert_eq!(
        ctx_ping.db.ping().await,
        Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
    );

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(base_url);
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(10));

    let db = sea_orm::Database::connect(opt).await.unwrap();

    async fn transaction_blocked(db: &DatabaseConnection) {
        let _txn = sea_orm::TransactionTrait::begin(db).await.unwrap();
        // Occupy the only connection, thus forcing others fail to acquire connection
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }

    async fn transaction(db: &DatabaseConnection) {
        // Should fail to acquire
        let txn = sea_orm::TransactionTrait::begin(db).await;
        assert_eq!(
            txn.expect_err("should be a time out"),
            crate::DbErr::ConnectionAcquire(ConnAcquireErr::Timeout)
        )
    }

    tokio::join!(transaction_blocked(&db), transaction(&db));

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
pub async fn connection_ping_closed_postgres() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed").await));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_postgres_connection_pool().close().await;
    assert_eq!(
        ctx_ping.db.ping().await,
        Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed))
    );

    let base_url = std::env::var("DATABASE_URL").unwrap();
    let mut opt = sea_orm::ConnectOptions::new(format!("{base_url}/connection_ping_closed"));
    opt
        // The connection pool has a single connection only
        .max_connections(1)
        // A controlled connection acquire timeout
        .acquire_timeout(std::time::Duration::from_secs(10));

    let db = sea_orm::Database::connect(opt).await.unwrap();

    async fn transaction_blocked(db: &DatabaseConnection) {
        let _txn = sea_orm::TransactionTrait::begin(db).await.unwrap();
        // Occupy the only connection, thus forcing others fail to acquire connection
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }

    async fn transaction(db: &DatabaseConnection) {
        // Should fail to acquire
        let txn = sea_orm::TransactionTrait::begin(db).await;
        assert_eq!(
            txn.expect_err("should be a time out"),
            crate::DbErr::ConnectionAcquire(ConnAcquireErr::Timeout)
        )
    }

    tokio::join!(transaction_blocked(&db), transaction(&db));

    ctx.delete().await;
}
