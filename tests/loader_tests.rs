#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
use sea_orm::{DbConn, DbErr, LoaderTraitEx, RuntimeErr, entity::*, query::*};

mod enum_pk_models {
    use crate::common::features::Tea;
    use sea_orm::entity::prelude::*;

    pub mod tea_inventory {
        use super::tea_order;
        use super::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "tea_inventory")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub tea: Tea,
            pub stock: i32,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(has_many = "super::tea_order::Entity")]
            TeaOrder,
        }

        impl Related<tea_order::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::TeaOrder.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    pub mod tea_order {
        use super::tea_inventory;
        use super::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "tea_orders")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub order_id: i32,
            #[sea_orm(primary_key, auto_increment = false)]
            pub tea: Tea,
            pub quantity: i32,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(
                belongs_to = "super::tea_inventory::Entity",
                from = "Column::Tea",
                to = "super::tea_inventory::Column::Tea"
            )]
            TeaInventory,
        }

        impl Related<tea_inventory::Entity> for Entity {
            fn to() -> RelationDef {
                Relation::TeaInventory.def()
            }
        }

        impl ActiveModelBehavior for ActiveModel {}
    }
}

use enum_pk_models::{tea_inventory, tea_order};

#[sea_orm_macros::test]
async fn loader_load_one() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_one").await;
    create_tables(&ctx.db).await?;

    let bakery_0 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Baker 1", bakery_0.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Baker 2", bakery_0.id).await?;
    let baker_3 = baker::ActiveModel {
        name: Set("Baker 3".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let bakeries = bakers.load_one(bakery::Entity, &ctx.db).await?;

    assert_eq!(bakers, [baker_1.clone(), baker_2.clone(), baker_3.clone()]);
    assert_eq!(
        bakeries,
        [Some(bakery_0.clone()), Some(bakery_0.clone()), None]
    );

    let bakers = vec![
        Some(baker_1.clone().into_ex()),
        None,
        Some(baker_2.clone().into_ex()),
        Some(baker_3.clone().into_ex()),
        None,
    ];
    let bakeries = bakers
        .as_slice()
        .load_one_ex(bakery::Entity, &ctx.db)
        .await?;
    assert_eq!(
        bakeries,
        [
            Some(bakery_0.clone()),
            None,
            Some(bakery_0.clone()),
            None,
            None
        ]
    );

    // has many find, should use load_many instead
    let bakeries = bakery::Entity::find().all(&ctx.db).await?;
    let bakers = bakeries.load_one(baker::Entity, &ctx.db).await;

    assert_eq!(
        bakers,
        Err(DbErr::Query(RuntimeErr::Internal(
            "Relation is HasMany instead of HasOne".to_string()
        )))
    );

    Ok(())
}

#[sea_orm_macros::test]
async fn loader_load_many() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;
    let bakery_2 = insert_bakery(&ctx.db, "Offshore Bakery").await?;
    let bakery_3 = insert_bakery(&ctx.db, "Rocky Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Baker 1", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Baker 2", bakery_1.id).await?;

    let baker_3 = insert_baker(&ctx.db, "John", bakery_2.id).await?;
    let baker_4 = insert_baker(&ctx.db, "Baker 4", bakery_2.id).await?;

    let bakeries = bakery::Entity::find().all(&ctx.db).await?;
    let bakers = bakeries.load_many(baker::Entity, &ctx.db).await?;

    assert_eq!(
        bakeries,
        [bakery_1.clone(), bakery_2.clone(), bakery_3.clone()]
    );
    assert_eq!(
        bakers,
        [
            vec![baker_1.clone(), baker_2.clone()],
            vec![baker_3.clone(), baker_4.clone()],
            vec![]
        ]
    );

    // test interlaced loader
    let bakeries_sparse = vec![
        Some(bakery_1.clone().into_ex()),
        None,
        Some(bakery_2.clone().into_ex()),
        None,
    ];
    let bakers = bakeries_sparse
        .as_slice()
        .load_many_ex(baker::Entity, &ctx.db)
        .await?;
    assert_eq!(
        bakers,
        [
            vec![baker_1.clone(), baker_2.clone()],
            vec![],
            vec![baker_3.clone(), baker_4.clone()],
            vec![],
        ]
    );

    // load bakers again but with additional condition

    let bakers = bakeries
        .load_many(
            baker::Entity::find().filter(baker::Column::Name.like("Baker%")),
            &ctx.db,
        )
        .await?;

    assert_eq!(
        bakers,
        [
            vec![baker_1.clone(), baker_2.clone()],
            vec![baker_4.clone()],
            vec![]
        ]
    );

    // now, start from baker

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let bakeries = bakers.load_one(bakery::Entity::find(), &ctx.db).await?;

    // note that two bakers share the same bakery
    assert_eq!(bakers, [baker_1, baker_2, baker_3, baker_4]);
    assert_eq!(
        bakeries,
        [
            Some(bakery_1.clone()),
            Some(bakery_1.clone()),
            Some(bakery_2.clone()),
            Some(bakery_2.clone())
        ]
    );

    // following should be equivalent
    let bakeries = bakers.load_many(bakery::Entity::find(), &ctx.db).await?;

    assert_eq!(
        bakeries,
        [
            vec![bakery_1.clone()],
            vec![bakery_1.clone()],
            vec![bakery_2.clone()],
            vec![bakery_2.clone()],
        ]
    );

    Ok(())
}

#[sea_orm_macros::test]
async fn loader_load_many_multi() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many_multi").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;
    let bakery_2 = insert_bakery(&ctx.db, "Offshore Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "John", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Jane", bakery_1.id).await?;
    let baker_3 = insert_baker(&ctx.db, "Peter", bakery_2.id).await?;

    let cake_1 = insert_cake(&ctx.db, "Cheesecake", Some(bakery_1.id)).await?;
    let cake_2 = insert_cake(&ctx.db, "Chocolate", Some(bakery_2.id)).await?;
    let cake_3 = insert_cake(&ctx.db, "Chiffon", Some(bakery_2.id)).await?;
    let _cake_4 = insert_cake(&ctx.db, "Apple Pie", None).await?; // no one makes apple pie

    let bakeries = bakery::Entity::find().all(&ctx.db).await?;
    let bakers = bakeries.load_many(baker::Entity, &ctx.db).await?;
    let cakes = bakeries.load_many(cake::Entity, &ctx.db).await?;

    assert_eq!(bakeries, [bakery_1, bakery_2]);
    assert_eq!(bakers, [vec![baker_1, baker_2], vec![baker_3]]);
    assert_eq!(cakes, [vec![cake_1], vec![cake_2, cake_3]]);

    Ok(())
}

#[sea_orm_macros::test]
async fn loader_load_many_enum_pk_postgres() -> Result<(), DbErr> {
    use crate::common::features::Tea;
    use sea_orm::{ActiveValue::Set, ConnectionTrait, DbBackend, Schema};
    use sea_query::{Alias, extension::postgres::Type};
    use tea_inventory::{
        ActiveModel as TeaInventoryActiveModel, Column as TeaInventoryColumn,
        Entity as TeaInventoryEntity, Model as TeaInventoryModel,
    };
    use tea_order::{
        ActiveModel as TeaOrderActiveModel, Column as TeaOrderColumn, Entity as TeaOrderEntity,
        Model as TeaOrderModel,
    };

    let ctx = TestContext::new("loader_enum_pk_postgres").await;
    let db = &ctx.db;

    if db.get_database_backend() != DbBackend::Postgres {
        return Ok(());
    }

    let schema = Schema::new(DbBackend::Postgres);

    let mut drop_type = Type::drop();
    drop_type.if_exists().name(Alias::new("tea")).cascade();
    db.execute(&drop_type).await?;

    let mut create_type = Type::create();
    create_type
        .as_enum(Alias::new("tea"))
        .values(["EverydayTea", "BreakfastTea"]);
    db.execute(&create_type).await?;

    let inventory_table = schema.create_table_from_entity(TeaInventoryEntity);
    create_table_without_asserts(db, &inventory_table).await?;
    let order_table = schema.create_table_from_entity(TeaOrderEntity);
    create_table_without_asserts(db, &order_table).await?;

    TeaInventoryActiveModel {
        tea: Set(Tea::EverydayTea),
        stock: Set(10),
    }
    .insert(db)
    .await?;
    TeaInventoryActiveModel {
        tea: Set(Tea::BreakfastTea),
        stock: Set(4),
    }
    .insert(db)
    .await?;

    TeaOrderActiveModel {
        order_id: Set(1),
        tea: Set(Tea::EverydayTea),
        quantity: Set(2),
    }
    .insert(db)
    .await?;
    TeaOrderActiveModel {
        order_id: Set(2),
        tea: Set(Tea::EverydayTea),
        quantity: Set(5),
    }
    .insert(db)
    .await?;
    TeaOrderActiveModel {
        order_id: Set(1),
        tea: Set(Tea::BreakfastTea),
        quantity: Set(1),
    }
    .insert(db)
    .await?;

    let teas = TeaInventoryEntity::find()
        .order_by_asc(TeaInventoryColumn::Tea)
        .all(db)
        .await?;
    let loaded_orders = teas
        .load_many(
            TeaOrderEntity::find().order_by_asc(TeaOrderColumn::OrderId),
            db,
        )
        .await?;

    assert_eq!(teas.len(), 2);
    assert_eq!(loaded_orders.len(), 2);

    for (inventory, orders) in teas.iter().zip(&loaded_orders) {
        match inventory.tea {
            Tea::EverydayTea => {
                assert_eq!(
                    orders,
                    &vec![
                        TeaOrderModel {
                            order_id: 1,
                            tea: Tea::EverydayTea,
                            quantity: 2,
                        },
                        TeaOrderModel {
                            order_id: 2,
                            tea: Tea::EverydayTea,
                            quantity: 5,
                        }
                    ]
                );
            }
            Tea::BreakfastTea => {
                assert_eq!(
                    orders,
                    &vec![TeaOrderModel {
                        order_id: 1,
                        tea: Tea::BreakfastTea,
                        quantity: 1,
                    }]
                );
            }
        }
    }

    Ok(())
}

#[sea_orm_macros::test]
async fn loader_load_many_to_many() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many_to_many").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Jane", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Peter", bakery_1.id).await?;
    let baker_3 = insert_baker(&ctx.db, "Fred", bakery_1.id).await?; // does not make cake

    let cake_1 = insert_cake(&ctx.db, "Cheesecake", None).await?;
    let cake_2 = insert_cake(&ctx.db, "Coffee", None).await?;
    let cake_3 = insert_cake(&ctx.db, "Chiffon", None).await?;
    let cake_4 = insert_cake(&ctx.db, "Apple Pie", None).await?; // no one makes apple pie

    insert_cake_baker(&ctx.db, baker_1.id, cake_1.id).await?;
    insert_cake_baker(&ctx.db, baker_1.id, cake_2.id).await?;
    insert_cake_baker(&ctx.db, baker_2.id, cake_2.id).await?;
    insert_cake_baker(&ctx.db, baker_2.id, cake_3.id).await?;

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let cakes = bakers
        .load_many_to_many(cake::Entity, cakes_bakers::Entity, &ctx.db)
        .await?;

    assert_eq!(bakers, [baker_1.clone(), baker_2.clone(), baker_3.clone()]);
    assert_eq!(
        cakes,
        [
            vec![cake_1.clone(), cake_2.clone()],
            vec![cake_2.clone(), cake_3.clone()],
            vec![]
        ]
    );

    // same, but apply restrictions on cakes

    let cakes = bakers
        .load_many_to_many(
            cake::Entity::find().filter(cake::Column::Name.like("Ch%")),
            cakes_bakers::Entity,
            &ctx.db,
        )
        .await?;
    assert_eq!(cakes, [vec![cake_1.clone()], vec![cake_3.clone()], vec![]]);

    // now, start again from cakes

    let cakes = cake::Entity::find().all(&ctx.db).await?;
    let bakers = cakes
        .load_many_to_many(baker::Entity, cakes_bakers::Entity, &ctx.db)
        .await?;

    assert_eq!(cakes, [cake_1, cake_2, cake_3, cake_4]);
    assert_eq!(
        bakers,
        [
            vec![baker_1.clone()],
            vec![baker_1.clone(), baker_2.clone()],
            vec![baker_2.clone()],
            vec![]
        ]
    );

    Ok(())
}

#[sea_orm_macros::test]
async fn loader_load_many_to_many_dyn() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many_to_many_dyn").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Jane", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Peter", bakery_1.id).await?;
    let baker_3 = insert_baker(&ctx.db, "Fred", bakery_1.id).await?; // does not make cake

    let cake_1 = insert_cake(&ctx.db, "Cheesecake", None).await?;
    let cake_2 = insert_cake(&ctx.db, "Coffee", None).await?;
    let cake_3 = insert_cake(&ctx.db, "Chiffon", None).await?;
    let cake_4 = insert_cake(&ctx.db, "Apple Pie", None).await?; // no one makes apple pie

    insert_cake_baker(&ctx.db, baker_1.id, cake_1.id).await?;
    insert_cake_baker(&ctx.db, baker_1.id, cake_2.id).await?;
    insert_cake_baker(&ctx.db, baker_2.id, cake_2.id).await?;
    insert_cake_baker(&ctx.db, baker_2.id, cake_3.id).await?;

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let cakes = bakers.load_many(cake::Entity, &ctx.db).await?;

    assert_eq!(bakers, [baker_1.clone(), baker_2.clone(), baker_3.clone()]);
    assert_eq!(
        cakes,
        [
            vec![cake_1.clone(), cake_2.clone()],
            vec![cake_2.clone(), cake_3.clone()],
            vec![]
        ]
    );

    // same, but apply restrictions on cakes

    let cakes = bakers
        .load_many_to_many(
            cake::Entity::find().filter(cake::Column::Name.like("Ch%")),
            cakes_bakers::Entity,
            &ctx.db,
        )
        .await?;
    assert_eq!(cakes, [vec![cake_1.clone()], vec![cake_3.clone()], vec![]]);

    // now, start again from cakes

    let cakes = cake::Entity::find().all(&ctx.db).await?;
    let bakers = cakes.load_many(baker::Entity, &ctx.db).await?;

    assert_eq!(cakes, [cake_1, cake_2, cake_3, cake_4]);
    assert_eq!(
        bakers,
        [
            vec![baker_1.clone()],
            vec![baker_1.clone(), baker_2.clone()],
            vec![baker_2.clone()],
            vec![]
        ]
    );

    Ok(())
}

pub async fn insert_bakery(db: &DbConn, name: &str) -> Result<bakery::Model, DbErr> {
    bakery::ActiveModel {
        name: Set(name.to_owned()),
        profit_margin: Set(1.0),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn insert_baker(db: &DbConn, name: &str, bakery_id: i32) -> Result<baker::Model, DbErr> {
    baker::ActiveModel {
        name: Set(name.to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(Some(bakery_id)),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn insert_cake(
    db: &DbConn,
    name: &str,
    bakery_id: Option<i32>,
) -> Result<cake::Model, DbErr> {
    cake::ActiveModel {
        name: Set(name.to_owned()),
        price: Set(rust_decimal::Decimal::ONE),
        gluten_free: Set(false),
        bakery_id: Set(bakery_id),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn insert_cake_baker(
    db: &DbConn,
    baker_id: i32,
    cake_id: i32,
) -> Result<cakes_bakers::Model, DbErr> {
    cakes_bakers::ActiveModel {
        cake_id: Set(cake_id),
        baker_id: Set(baker_id),
    }
    .insert(db)
    .await
}
