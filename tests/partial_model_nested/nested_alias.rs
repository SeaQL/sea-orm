use crate::common::TestContext;
use crate::local::{bakery, create_tables, worker};
use sea_orm::{
    DbBackend, DerivePartialModel, FromQueryResult, IntoActiveModel, JoinType, NotSet, QueryOrder,
    QuerySelect, QueryTrait, Set, prelude::*, sea_query::Alias,
};

#[derive(DerivePartialModel)]
#[sea_orm(entity = "worker::Entity")]
struct Worker {
    id: i32,
    name: String,
}

#[derive(DerivePartialModel)]
#[sea_orm(entity = "bakery::Entity")]
struct BakeryWorker {
    id: i32,
    name: String,
    profit_margin: f64,
    #[sea_orm(nested, alias = "manager")]
    manager: Worker,
    #[sea_orm(nested, alias = "cashier")]
    cashier: worker::Model,
}

#[derive(DerivePartialModel)]
#[sea_orm(entity = "worker::Entity")]
struct ManagerOfBakery {
    id: i32,
    name: String,
    #[sea_orm(nested)]
    bakery: bakery::Model,
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
        name: Set("Master Bakery".to_string()),
        profit_margin: Set(4.1),
        manager_id: Set(1),
        cashier_id: Set(2),
    })
    .exec(&ctx.db)
    .await
    .expect("insert succeeds");

    let selector = bakery::Entity::find()
        .join_as(
            sea_orm::JoinType::LeftJoin,
            bakery::Relation::Manager.def(),
            "manager",
        )
        .join_as(
            sea_orm::JoinType::LeftJoin,
            bakery::Relation::Cashier.def(),
            "cashier",
        );

    assert_eq!(
        selector.build(DbBackend::MySql).to_string(),
        "SELECT `bakery`.`id`, `bakery`.`name`, `bakery`.`profit_margin`, `bakery`.`manager_id`, `bakery`.`cashier_id` FROM `bakery` LEFT JOIN `worker` AS `manager` ON `bakery`.`manager_id` = `manager`.`id` LEFT JOIN `worker` AS `cashier` ON `bakery`.`cashier_id` = `cashier`.`id`"
    );

    let bakery: BakeryWorker = selector
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(bakery.manager.name, "Tom");
    assert_eq!(bakery.cashier.name, "Jerry");

    let selector = worker::Entity::find().join(
        sea_orm::JoinType::LeftJoin,
        worker::Relation::BakeryManager.def(),
    );

    assert_eq!(
        selector.build(DbBackend::MySql).to_string(),
        "SELECT `worker`.`id`, `worker`.`name` FROM `worker` LEFT JOIN `bakery` ON `worker`.`id` = `bakery`.`manager_id`"
    );

    let manager: ManagerOfBakery = selector
        .into_partial_model()
        .one(&ctx.db)
        .await
        .expect("succeeds to get the result")
        .expect("exactly one model in DB");

    assert_eq!(manager.name, "Tom");
    assert_eq!(manager.bakery.name, "Master Bakery");

    ctx.delete().await;
}
