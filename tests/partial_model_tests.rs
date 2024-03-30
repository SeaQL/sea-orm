#![allow(unused_imports, dead_code)]

use entity::{Column, Entity};
use sea_orm::{prelude::*, DerivePartialModel, FromQueryResult, Set};

use crate::common::TestContext;

mod common;

mod entity {
    use sea_orm::prelude::*;

    #[derive(Debug, Clone, DeriveEntityModel)]
    #[sea_orm(table_name = "foo_table")]
    pub struct Model {
        #[sea_orm(primary_key)]
        id: i32,
        foo: i32,
        bar: String,
        foo2: bool,
        bar2: f64,
    }

    #[derive(Debug, DeriveRelation, EnumIter)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct SimpleTest {
    _foo: i32,
    _bar: String,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "<entity::Model as ModelTrait>::Entity")]
struct EntityNameNotAIdent {
    #[sea_orm(from_col = "foo2")]
    _foo: i32,
    #[sea_orm(from_col = "bar2")]
    _bar: String,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct FieldFromDiffNameColumnTest {
    #[sea_orm(from_col = "foo2")]
    _foo: i32,
    #[sea_orm(from_col = "bar2")]
    _bar: String,
}

#[derive(FromQueryResult, DerivePartialModel)]
struct FieldFromExpr {
    #[sea_orm(from_expr = "Column::Bar2.sum()")]
    _foo: f64,
    #[sea_orm(from_expr = "Expr::col(Column::Id).equals(Column::Foo)")]
    _bar: bool,
}

#[derive(FromQueryResult, DerivePartialModel)]
struct Nest {
    #[sea_orm(nested)]
    _foo: SimpleTest,
}

#[derive(FromQueryResult, DerivePartialModel)]
struct NestOption {
    #[sea_orm(nested)]
    _foo: Option<SimpleTest>,
}

use common::bakery_chain::*;

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "bakery::Entity")]
struct Bakery {
    _id: i32,
    #[sea_orm(from_col = "Name")]
    _title: String,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "bakery::Entity")]
struct BakeryDetails {
    #[sea_orm(nested)]
    _basics: Bakery,
    _profit_margin: f64,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "cake::Entity")]
struct Cake {
    _id: i32,
    _name: String,
    #[sea_orm(nested)]
    _bakery: Option<Bakery>,
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
async fn partial_model_left_join_does_not_exist() {
    let ctx = TestContext::new("partial_model_left_join_does_not_exist").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, false).await;

    let cake: Cake = cake::Entity::find()
        .left_join(bakery::Entity)
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(cake._id, 13);
    assert!(cake._bakery.is_none());

    ctx.delete().await;
}

#[sea_orm_macros::test]
async fn partial_model_left_join_exists() {
    let ctx = TestContext::new("partial_model_left_join_exists").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, true).await;

    let cake: Cake = cake::Entity::find()
        .left_join(bakery::Entity)
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(cake._id, 13);
    assert!(matches!(cake._bakery, Some(Bakery { _id: 42, .. })));

    ctx.delete().await;
}

#[sea_orm_macros::test]
async fn partial_model_nested_same_table() {
    let ctx = TestContext::new("partial_model_nested_same_table").await;
    create_tables(&ctx.db).await.unwrap();

    fill_data(&ctx, true).await;

    let bakery: BakeryDetails = bakery::Entity::find()
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(bakery._basics._id, 42);

    ctx.delete().await;
}
