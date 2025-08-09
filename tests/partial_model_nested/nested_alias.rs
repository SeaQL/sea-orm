use crate::common::TestContext;
use crate::local::{bakery, create_tables, worker};
use sea_orm::{
    prelude::*, sea_query::Alias, DerivePartialModel, FromQueryResult, IntoActiveModel, JoinType,
    NotSet, QueryOrder, QuerySelect, Set,
};

#[derive(DerivePartialModel)]
#[sea_orm(entity = "worker::Entity", from_query_result)]
struct Worker {
    id: i32,
    name: String,
}
#[derive(DerivePartialModel)]
#[sea_orm(entity = "bakery::Entity", from_query_result)]
struct BakeryWorker {
    id: i32,
    name: String,
    profit_margin: f64,
    #[sea_orm(nested, alias = "manager")]
    manager: Worker,
    #[sea_orm(nested, alias = "cashier")]
    cashier: Worker,
}

#[sea_orm_macros::test]
async fn partial_model_nested_alias() {
    let ctx = TestContext::new("partial_model_nested").await;
    create_tables(&ctx.db)
        .await
        .expect("unable to create tables");

    // TODO: init utils

    worker::Entity::insert(worker::ActiveModel {
        id: Set(1),
        name: Set("Tom".to_owned()),
        ..Default::default()
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");

    worker::Entity::insert(worker::ActiveModel {
        id: Set(2),
        name: Set("Jerry".to_owned()),
        ..Default::default()
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");

    bakery::Entity::insert(bakery::ActiveModel {
        id: Set(42),
        name: Set("cool little bakery".to_string()),
        profit_margin: Set(4.1),
        manager_id: Set(1),
        cashier_id: Set(2),
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");

    let bakery: BakeryWorker = bakery::Entity::find()
        .join_as(
            sea_orm::JoinType::LeftJoin,
            bakery::Relation::Manager.def(),
            "manager",
        )
        .join_as(
            sea_orm::JoinType::LeftJoin,
            bakery::Relation::Cashier.def(),
            "cashier",
        )
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(bakery.manager.name, "Tom");
    assert_eq!(bakery.cashier.name, "Jerry");

    ctx.delete().await;
}
