#![allow(unused_imports, dead_code)]

use sea_orm::{FromQueryResult, prelude::*, raw_sql};

use crate::common::TestContext;
use common::bakery_chain::*;
use serde_json::json;

mod common;

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-sqlite")]
async fn raw_sql_test_simple_select() {
    #[derive(FromQueryResult)]
    struct BakerySimple {
        id: i32,
        brand: String,
    }

    #[derive(FromQueryResult)]
    struct BakeryFlat {
        id: i32,
        name: String,
        #[sea_orm(alias = "profit_margin")]
        profit: f64,
    }

    let ctx = TestContext::new("raw_sql_test_simple_select").await;
    create_tables(&ctx.db).await.unwrap();

    seed_data::init_1(&ctx, false).await;

    let id = 42;

    let bakery: bakery::Model = bakery::Entity::find()
        .from_raw_sql(raw_sql!(
            Sqlite,
            r#"SELECT "id", "name", "profit_margin" FROM "bakery" WHERE id = {id}"#
        ))
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(bakery.id, 42);
    assert_eq!(bakery.name, "cool little bakery");
    assert_eq!(bakery.profit_margin, 4.1);

    let bakery = BakeryFlat::find_by_statement(raw_sql!(
        Sqlite,
        r#"SELECT "id", "name", "profit_margin" FROM "bakery" WHERE id = {id}"#
    ))
    .one(&ctx.db)
    .await
    .expect("succeeds to get the result")
    .expect("exactly one model in DB");

    assert_eq!(bakery.id, 42);
    assert_eq!(bakery.name, "cool little bakery");
    assert_eq!(bakery.profit, 4.1);

    let bakery = BakerySimple::find_by_statement(raw_sql!(
        Sqlite,
        r#"SELECT "id", "name" as "brand" FROM "bakery" WHERE id = {id}"#
    ))
    .one(&ctx.db)
    .await
    .expect("succeeds to get the result")
    .expect("exactly one model in DB");

    assert_eq!(bakery.id, 42);
    assert_eq!(bakery.brand, "cool little bakery");

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-sqlite")]
async fn raw_sql_test_nested_select() {
    #[derive(FromQueryResult)]
    struct Cake {
        id: i32,
        name: String,
        #[sea_orm(nested)]
        bakery: Option<Bakery>,
    }

    #[derive(FromQueryResult)]
    struct Bakery {
        #[sea_orm(alias = "bakery_id")]
        id: i32,
        #[sea_orm(alias = "bakery_name")]
        name: String,
    }

    let ctx = TestContext::new("raw_sql_test_nested_select").await;
    create_tables(&ctx.db).await.unwrap();

    seed_data::init_1(&ctx, true).await;

    let bakery_id = 42;
    let cake_ids = [10, 12, 15];

    let cake: Option<Cake> = Cake::find_by_statement(raw_sql!(
        Sqlite,
        r#"SELECT
                "cake"."id",
                "cake"."name",
                "bakery"."id" AS "bakery_id",
                "bakery"."name" AS "bakery_name"
            FROM "cake"
            LEFT JOIN "bakery" ON "cake"."bakery_id" = "bakery"."id"
            WHERE
                "bakery"."id" = {bakery_id}
                AND "cake"."id" IN ({..cake_ids})
            ORDER BY "cake"."id" ASC LIMIT 1"#
    ))
    .one(&ctx.db)
    .await
    .expect("succeeds to get the result");

    let cake = cake.unwrap();
    assert_eq!(cake.id, 15);
    assert_eq!(cake.name, "Chocolate");
    let bakery = cake.bakery.unwrap();
    assert_eq!(bakery.id, 42);
    assert_eq!(bakery.name, "cool little bakery");

    ctx.delete().await;
}
