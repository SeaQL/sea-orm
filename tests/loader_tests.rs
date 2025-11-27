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

    let mut drop_type = Type::drop();
    drop_type.if_exists().name("tea").cascade();
    db.execute(&drop_type).await?;

    db.get_schema_builder()
        .register(TeaInventoryEntity)
        .register(TeaOrderEntity)
        .apply(db)
        .await?;

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
            Tea::AfternoonTea => {}
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

#[sea_orm_macros::test]
async fn loader_self_join() -> Result<(), DbErr> {
    use common::film_store::{staff, staff_compact};
    use sea_orm::tests_cfg::{user, user_follower};

    let ctx = TestContext::new("test_loader_self_join").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(staff::Entity)
        .register(user::Entity)
        .register(user_follower::Entity)
        .apply(db)
        .await?;

    let alan = staff::ActiveModel {
        name: Set("Alan".into()),
        reports_to_id: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    staff::ActiveModel {
        name: Set("Ben".into()),
        reports_to_id: Set(Some(alan.id)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    staff::ActiveModel {
        name: Set("Alice".into()),
        reports_to_id: Set(Some(alan.id)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    staff::ActiveModel {
        name: Set("Elle".into()),
        reports_to_id: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let staff = staff::Entity::find()
        .order_by_asc(staff::Column::Id)
        .all(db)
        .await?;

    let reports_to = staff
        .load_self(staff::Entity, staff::Relation::ReportsTo, db)
        .await?;

    assert_eq!(staff[0].name, "Alan");
    assert_eq!(reports_to[0], None);

    assert_eq!(staff[1].name, "Ben");
    assert_eq!(reports_to.get(1).unwrap().as_ref().unwrap().name, "Alan");

    assert_eq!(staff[2].name, "Alice");
    assert_eq!(reports_to.get(2).unwrap().as_ref().unwrap().name, "Alan");

    assert_eq!(staff[3].name, "Elle");
    assert_eq!(reports_to[3], None);

    let manages = staff
        .load_self_many(
            staff::Entity::find().order_by_asc(staff::COLUMN.id),
            staff::Relation::ReportsTo,
            db,
        )
        .await?;

    assert_eq!(
        manages,
        staff
            .load_self_many(
                staff::Entity::find().order_by_asc(staff::COLUMN.id),
                staff::Relation::Manages,
                db,
            )
            .await?
    );

    assert_eq!(staff[0].name, "Alan");
    assert_eq!(manages[0].len(), 2);
    assert_eq!(manages[0][0].name, "Ben");
    assert_eq!(manages[0][1].name, "Alice");

    assert_eq!(staff[1].name, "Ben");
    assert_eq!(manages[1].len(), 0);

    assert_eq!(staff[2].name, "Alice");
    assert_eq!(manages[2].len(), 0);

    assert_eq!(staff[3].name, "Elle");
    assert_eq!(manages[3].len(), 0);

    // test self_ref on compact_model

    let staff = staff_compact::Entity::find()
        .order_by_asc(staff_compact::COLUMN.id)
        .all(db)
        .await?;

    let reports_to = staff
        .load_self(
            staff_compact::Entity,
            staff_compact::Relation::ReportsTo,
            db,
        )
        .await?;

    let manages = staff
        .load_self_many(
            staff_compact::Entity::find().order_by_asc(staff_compact::COLUMN.id),
            staff_compact::Relation::ReportsTo,
            db,
        )
        .await?;

    assert_eq!(
        manages,
        staff
            .load_self_many(
                staff_compact::Entity::find().order_by_asc(staff_compact::COLUMN.id),
                staff_compact::Relation::Manages,
                db,
            )
            .await?
    );

    assert_eq!(staff[0].name, "Alan");
    assert_eq!(reports_to[0], None);
    assert_eq!(manages[0].len(), 2);
    assert_eq!(manages[0][0].name, "Ben");
    assert_eq!(manages[0][1].name, "Alice");

    assert_eq!(staff[1].name, "Ben");
    assert_eq!(reports_to.get(1).unwrap().as_ref().unwrap().name, "Alan");
    assert_eq!(manages[1].len(), 0);

    assert_eq!(staff[2].name, "Alice");
    assert_eq!(reports_to.get(2).unwrap().as_ref().unwrap().name, "Alan");
    assert_eq!(manages[2].len(), 0);

    assert_eq!(staff[3].name, "Elle");
    assert_eq!(reports_to[3], None);
    assert_eq!(manages[3].len(), 0);

    // self_ref + via

    let alice = user::ActiveModel {
        name: Set("Alice".into()),
        email: Set("@1".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let bob = user::ActiveModel {
        name: Set("Bob".into()),
        email: Set("@2".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let sam = user::ActiveModel {
        name: Set("Sam".into()),
        email: Set("@3".into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    user_follower::ActiveModel {
        user_id: Set(alice.id),
        follower_id: Set(bob.id),
    }
    .insert(db)
    .await?;

    user_follower::ActiveModel {
        user_id: Set(alice.id),
        follower_id: Set(sam.id),
    }
    .insert(db)
    .await?;

    user_follower::ActiveModel {
        user_id: Set(bob.id),
        follower_id: Set(sam.id),
    }
    .insert(db)
    .await?;

    let users = user::Entity::find().all(db).await?;
    let followers = users.load_self_via(user_follower::Entity, db).await?;
    assert_eq!(users[0], alice);
    assert_eq!(users[1], bob);
    assert_eq!(users[2], sam);
    assert_eq!(followers[0], [bob, sam.clone()]);
    assert_eq!(followers[1], [sam]);
    assert!(followers[2].is_empty());

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
