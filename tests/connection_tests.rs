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
    assert_eq!(ctx_ping.db.ping().await, Err(DbErr::ConnectionAcquire));
    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-sqlite")]
pub async fn connection_ping_closed_sqlite() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed").await));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_sqlite_connection_pool().close().await;
    assert_eq!(ctx_ping.db.ping().await, Err(DbErr::ConnectionAcquire));
    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
pub async fn connection_ping_closed_postgres() {
    let ctx = std::rc::Rc::new(Box::new(TestContext::new("connection_ping_closed").await));
    let ctx_ping = std::rc::Rc::clone(&ctx);

    ctx.db.get_postgres_connection_pool().close().await;
    assert_eq!(ctx_ping.db.ping().await, Err(DbErr::ConnectionAcquire));
    ctx.delete().await;
}
