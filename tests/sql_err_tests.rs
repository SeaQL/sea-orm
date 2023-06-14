pub mod common;
mod crud;
pub use sea_orm::entity::*;
pub use sea_orm::{ConnectionTrait, QueryFilter, QuerySelect, sea_query, tests_cfg, EntityName};
use rust_decimal_macros::dec;
use sea_orm::error::*;
// use sea_query::ForeignKey;
use uuid::Uuid;

pub use common::{bakery_chain::*, setup::*, TestContext};
use pretty_assertions::assert_eq;

pub use crud::*;
use sea_orm::DatabaseConnection;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() {
    let ctx = TestContext::new("bakery_chain_sql_err_tests").await;
    create_tables(&ctx.db).await.unwrap();
    test_error(&ctx.db).await;
    ctx.delete().await;

    // let ctx = TestContext::new("bakery_chain_sql_err_tests_2").await;
    // create_tables(&ctx.db2).await.unwrap();
    // test_error_foreign(&ctx.db1, &ctx.db2).await;
    // ctx.delete().await;
}

pub async fn test_error(db: &DatabaseConnection) {
    let mud_cake = cake::ActiveModel {
        name: Set("Moldy Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(None),
        ..Default::default()
    };

    let cake = mud_cake.save(db).await.expect("could not insert cake");

    // if compiling without sqlx, this assignment will complain,
    // but the whole test is useless in that case anyway.
    #[allow(unused_variables)]
    let error: DbErr = cake
        .into_active_model()
        .insert(db)
        .await
        .expect_err("inserting should fail due to duplicate primary key");

    let sqlerr = error.sql_err();
    assert_eq!(sqlerr.unwrap(), SqlErr::UniqueConstraintViolation())
}

// pub async fn test_error_foreign(db1: &DatabaseConnection, db2: &DatabaseConnection) {
    

    
// }