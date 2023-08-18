pub mod common;
mod crud;

pub use common::{bakery_chain::*, setup::*, TestContext};
pub use sea_orm::{
    entity::*, error::DbErr, tests_cfg, DatabaseConnection, DbBackend, EntityName, ExecResult,
};

pub use crud::*;
// use common::bakery_chain::*;
use sea_orm::{DbConn, TryInsertResult};
use sea_query::OnConflict;

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

    assert!(matches!(res, Ok(TryInsertResult::Inserted(_))));

    let double_seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        id: Set(1),
    };

    let empty_insert = Bakery::insert_many(std::iter::empty::<bakery::ActiveModel>())
        .on_empty_do_nothing()
        .exec(db)
        .await;

    assert!(matches!(empty_insert, Ok(TryInsertResult::Empty)));
}
