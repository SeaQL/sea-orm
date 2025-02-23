#![allow(unused_imports, dead_code)]

use sea_orm::{prelude::*, query::QuerySelect, FromQueryResult, Set};

use crate::common::TestContext;
use common::bakery_chain::*;

mod common;

#[derive(FromQueryResult)]
#[sea_orm(entity = "cake::Entity")]
struct Cake {
    id: i32,
    name: String,
    #[sea_orm(nested)]
    bakery: Option<CakeBakery>,
}

#[derive(FromQueryResult)]
struct CakeBakery {
    #[sea_orm(from_alias = "bakery_id")]
    id: i32,
    #[sea_orm(from_alias = "bakery_name")]
    title: String,
}

#[derive(FromQueryResult)]
struct BakeryDetails {
    #[sea_orm(nested)]
    basics: Bakery,
    profit: f64,
}

#[derive(FromQueryResult)]
struct Bakery {
    id: i32,
    title: String,
}

#[derive(FromQueryResult)]
struct BakeryFlat {
    id: i32,
    name: String,
    #[sea_orm(from_alias = "profit_margin")]
    profit: f64,
}

async fn fill_data(ctx: &TestContext, link: bool) {
    bakery::Entity::insert(bakery::ActiveModel {
        id: Set(42),
        name: Set("cool little bakery".to_string()),
        profit_margin: Set(4.1),
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");

    cake::Entity::insert(cake::ActiveModel {
        id: Set(13),
        name: Set("Test Cake".to_owned()),
        price: Set(Decimal::ZERO),
        bakery_id: Set(if link { Some(42) } else { None }),
        gluten_free: Set(true),
        serial: Set(Uuid::new_v4()),
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");
}

#[sea_orm_macros::test]
async fn from_query_result_left_join_does_not_exist() {
    let ctx = TestContext::new("from_query_result_left_join_does_not_exist").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, false).await;

    let cake: Cake = cake::Entity::find()
        .select_only()
        .column(cake::Column::Id)
        .column(cake::Column::Name)
        .column_as(bakery::Column::Id, "bakery_id")
        .column_as(bakery::Column::Name, "bakery_name")
        .left_join(bakery::Entity)
        .into_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(cake.id, 13);
    assert_eq!(cake.name, "Test Cake");
    assert!(cake.bakery.is_none());

    ctx.delete().await;
}

#[sea_orm_macros::test]
async fn from_query_result_left_join_exists() {
    let ctx = TestContext::new("from_query_result_left_join_exists").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, true).await;

    let cake: Cake = cake::Entity::find()
        .select_only()
        .column(cake::Column::Id)
        .column(cake::Column::Name)
        .column_as(bakery::Column::Id, "bakery_id")
        .column_as(bakery::Column::Name, "bakery_name")
        .left_join(bakery::Entity)
        .into_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(cake.id, 13);
    assert_eq!(cake.name, "Test Cake");
    let bakery = cake.bakery.unwrap();
    assert_eq!(bakery.id, 42);
    assert_eq!(bakery.title, "cool little bakery");

    ctx.delete().await;
}

#[sea_orm_macros::test]
async fn from_query_result_flat() {
    let ctx = TestContext::new("from_query_result_flat").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, true).await;

    let bakery: BakeryFlat = bakery::Entity::find()
        .into_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(bakery.id, 42);
    assert_eq!(bakery.name, "cool little bakery");
    assert_eq!(bakery.profit, 4.1);

    ctx.delete().await;
}

#[sea_orm_macros::test]
async fn from_query_result_nested() {
    let ctx = TestContext::new("from_query_result_nested").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, true).await;

    let bakery: BakeryDetails = bakery::Entity::find()
        .select_only()
        .column(bakery::Column::Id)
        .column_as(bakery::Column::Name, "title")
        .column_as(bakery::Column::ProfitMargin, "profit")
        .into_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(bakery.basics.id, 42);
    assert_eq!(bakery.basics.title, "cool little bakery");
    assert_eq!(bakery.profit, 4.1);

    ctx.delete().await;
}

#[derive(Debug, FromQueryResult)]
struct WrongBakery {
    id: String,
    title: String,
}

#[derive(Debug, FromQueryResult)]
struct WrongCake {
    id: i32,
    name: String,
    #[sea_orm(nested)]
    bakery: Option<WrongBakery>,
}

#[sea_orm_macros::test]
async fn from_query_result_optional_field_but_type_error() {
    let ctx = TestContext::new("from_query_result_nested_error").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, false).await;

    let _: DbErr = cake::Entity::find()
        .select_only()
        .column(cake::Column::Id)
        .left_join(bakery::Entity)
        .into_model::<WrongCake>()
        .one(&ctx.db)
        .await
        .expect_err("should error instead of returning an empty Option");

    ctx.delete().await;
}
