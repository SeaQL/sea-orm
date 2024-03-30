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

#[sea_orm_macros::test]
async fn partial_model_left_join_does_not_exist() {
    use common::bakery_chain::*;

    #[derive(FromQueryResult, DerivePartialModel)]
    #[sea_orm(entity = "bakery::Entity")]
    struct Bakery {
        id: i32,
        name: String,
    }

    #[derive(FromQueryResult, DerivePartialModel)]
    #[sea_orm(entity = "cake::Entity")]
    struct Cake {
        id: i32,
        name: String,
        #[sea_orm(nested)]
        bakery: Option<Bakery>,
    }

    let ctx = TestContext::new("find_one_with_result").await;
    create_tables(&ctx.db).await.unwrap();

    cake::Entity::insert(cake::ActiveModel {
        name: Set("Test Cake".to_owned()),
        price: Set(Decimal::ZERO),
        bakery_id: Set(None),
        gluten_free: Set(true),
        serial: Set(Uuid::new_v4()),
        ..Default::default()
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");

    let data: Cake = cake::Entity::find()
        .left_join(bakery::Entity)
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert!(data.bakery.is_none());

    ctx.delete().await;
}
