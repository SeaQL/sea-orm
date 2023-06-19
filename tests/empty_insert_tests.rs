pub mod common;
mod crud;

pub use common::{bakery_chain::*, setup::*, TestContext};
pub use sea_orm::{
    entity::*, error::DbErr, tests_cfg, DatabaseConnection, DbBackend, EntityName, ExecResult,
};

pub use crud::*;
// use common::bakery_chain::*;
use sea_orm::{DbConn, TryInsertResult};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() {
    let ctx = TestContext::new("bakery_chain_empty_insert_tests").await;
    create_tables(&ctx.db).await.unwrap();
    test(&ctx.db).await;
    ctx.delete().await;
}

pub async fn test(db: &DbConn) {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };

    let res = Bakery::insert(seaside_bakery)
        .on_empty_do_nothing()
        .exec(db)
        .await;

    assert!(matches!(res, TryInsertResult::Inserted(_)));

    let empty_iterator = [bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }]
    .into_iter()
    .filter(|_| false);

    let empty_insert = Bakery::insert_many(empty_iterator)
        .on_empty_do_nothing()
        .exec(db)
        .await;

    assert!(matches!(empty_insert, TryInsertResult::Empty));
}
