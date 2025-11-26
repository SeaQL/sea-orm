#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{
    TestContext,
    bakery_chain::{create_tables, seed_data},
    bakery_dense::{prelude::*, *},
    setup::*,
};
use sea_orm::{DbConn, DbErr, EntityLoaderTrait, RuntimeErr, Set, prelude::*, query::*};

#[sea_orm_macros::test]
async fn cake_entity_loader() -> Result<(), DbErr> {
    use sea_orm::compound::EntityLoaderTrait;

    let ctx = TestContext::new("test_cake_entity_loader").await;
    let db = &ctx.db;
    create_tables(db).await?;

    let bakery_1 = insert_bakery(db, "SeaSide Bakery").await?;
    let bakery_2 = insert_bakery(db, "LakeSide Bakery").await?;

    let baker_1 = insert_baker(db, "Jane", bakery_1.id).await?;
    let baker_2 = insert_baker(db, "Peter", bakery_1.id).await?;
    let baker_3 = insert_baker(db, "Fred", bakery_2.id).await?; // does not make cake

    let cake_1 = insert_cake(db, "Cheesecake", Some(bakery_1.id)).await?;
    let cake_2 = insert_cake(db, "Coffee", Some(bakery_1.id)).await?;
    let cake_3 = insert_cake(db, "Chiffon", Some(bakery_2.id)).await?;
    let cake_4 = insert_cake(db, "Apple Pie", None).await?; // no one makes apple pie

    insert_cake_baker(db, baker_1.id, cake_1.id).await?;
    insert_cake_baker(db, baker_1.id, cake_2.id).await?;
    insert_cake_baker(db, baker_2.id, cake_2.id).await?;
    insert_cake_baker(db, baker_2.id, cake_3.id).await?;

    let cakes = Cake::load().all(db).await?;
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone(),
        ]
    );

    let cakes = Cake::load()
        .filter(Cake::COLUMN.name.like("Ch%"))
        .all(db)
        .await?;
    assert_eq!(cakes, [cake_1.clone(), cake_3.clone()]);
    assert!(cakes[0].bakers.is_empty());
    assert!(cakes[0].bakery.is_unloaded());

    assert_eq!(
        Cake::load()
            .filter(Cake::COLUMN.name.like("Ch%"))
            .order_by_desc(Cake::COLUMN.name)
            .one(db)
            .await?
            .unwrap(),
        cake_3
    );

    let cakes = Cake::load().with(Bakery).all(db).await?;
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone(),
        ]
    );
    assert_eq!(cakes[0].bakery.as_ref().unwrap(), &bakery_1);
    assert_eq!(cakes[1].bakery.as_ref().unwrap(), &bakery_1);
    assert_eq!(cakes[2].bakery.as_ref().unwrap(), &bakery_2);
    assert_eq!(cakes[3].bakery, None);

    // low-level API
    assert_eq!(
        cakes,
        cake::EntityLoader::load(
            Cake::load().all(db).await?,
            &cake::EntityLoaderWith {
                bakery: true,
                lineitems: false,
                bakers: false,
            },
            &Default::default(),
            db
        )
        .await?
    );

    let cake_with_bakery = Cake::load()
        .filter(Cake::COLUMN.name.eq("Cheesecake"))
        .with(Bakery)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(cake_with_bakery, cake_1);
    assert_eq!(cake_with_bakery.bakery.unwrap(), bakery_1);
    assert!(cake_with_bakery.bakers.is_empty());

    assert_eq!(
        Cake::load()
            .filter_by_id(cake_2.id)
            .with(Bakery)
            .one(db)
            .await?
            .unwrap(),
        {
            let mut cake_2 = cake_2.clone().into_ex();
            cake_2.bakery = HasOne::loaded(bakery_1.clone());
            cake_2
        }
    );

    let cakes = Cake::load().with(Bakery).with(Baker).all(db).await?;
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone()
        ]
    );
    assert_eq!(cakes[0].bakery.as_ref().unwrap(), &bakery_1);
    assert_eq!(cakes[1].bakery.as_ref().unwrap(), &bakery_1);
    assert_eq!(cakes[2].bakery.as_ref().unwrap(), &bakery_2);
    assert_eq!(cakes[3].bakery, None);
    assert_eq!(cakes[0].bakers, [baker_1.clone()]);
    assert_eq!(cakes[1].bakers, [baker_1.clone(), baker_2.clone()]);
    assert_eq!(cakes[2].bakers, [baker_2.clone()]);
    assert!(cakes[3].bakers.is_empty());

    let cake_with_bakery_baker = Cake::load()
        .filter(Cake::COLUMN.name.eq("Chiffon"))
        .with(Bakery)
        .with(Baker)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(cake_with_bakery_baker, cake_3);
    assert_eq!(cake_with_bakery_baker.bakery.unwrap(), bakery_2);
    assert_eq!(cake_with_bakery_baker.bakers, [baker_2.clone()]);

    // start again from baker

    let bakers = baker::Entity::find().all(db).await?;
    assert_eq!(bakers, [baker_1.clone(), baker_2.clone(), baker_3.clone()]);

    let bakers = baker::Entity::load().with(cake::Entity).all(db).await?;
    assert_eq!(bakers[0].id, baker_1.id);
    assert_eq!(bakers[1].id, baker_2.id);
    assert_eq!(bakers[2].id, baker_3.id);
    assert_eq!(bakers[0].cakes, [cake_1.clone(), cake_2.clone()]);
    assert_eq!(bakers[1].cakes, [cake_2.clone(), cake_3.clone()]);
    assert!(bakers[2].cakes.is_empty());

    // alternative API
    assert_eq!(
        bakers,
        baker::EntityLoader::load(
            baker::Entity::load().all(db).await?,
            &baker::EntityLoaderWith {
                cakes: true,
                bakery: false,
            },
            &Default::default(),
            db
        )
        .await?
    );

    // 2 many
    // bakery -> baker
    //        -> cake

    let bakeries = bakery::Entity::load()
        .with(Baker)
        .with(cake::Entity)
        .order_by_asc(bakery::Column::Id)
        .all(db)
        .await?;

    assert_eq!(bakeries[0].bakers, [baker_1.clone(), baker_2.clone()]);
    assert_eq!(bakeries[1].bakers, [baker_3.clone()]);
    assert_eq!(bakeries[0].cakes, [cake_1.clone(), cake_2.clone()]);
    assert_eq!(bakeries[1].cakes, [cake_3.clone()]);

    // nested
    // cake -> bakery -> baker

    let cakes = Cake::load().with((Bakery, Baker)).all(db).await?;
    assert_eq!(cakes[0].bakery.as_ref().unwrap().name, bakery_1.name);
    assert_eq!(cakes[1].bakery.as_ref().unwrap().name, bakery_1.name);
    assert_eq!(cakes[2].bakery.as_ref().unwrap().name, bakery_2.name);
    assert_eq!(cakes[3].bakery, None);
    assert_eq!(
        cakes[0].bakery.as_ref().unwrap().bakers,
        [baker_1.clone(), baker_2.clone()]
    );
    assert_eq!(
        cakes[1].bakery.as_ref().unwrap().bakers,
        [baker_1.clone(), baker_2.clone()]
    );
    assert_eq!(cakes[2].bakery.as_ref().unwrap().bakers, [baker_3.clone()]);

    Ok(())
}

#[sea_orm_macros::test]
async fn entity_loader_join_three() {
    let ctx = TestContext::new("entity_loader_join_three").await;
    create_tables(&ctx.db).await.unwrap();

    seed_data::init_1(&ctx, true).await;

    let db = &ctx.db;

    // verify basics
    let cake_13 = cake::Entity::find_by_id(13).one(db).await.unwrap().unwrap();
    let cake_15 = cake::Entity::find_by_id(15).one(db).await.unwrap().unwrap();

    // new find by key feature
    #[rustfmt::skip]
    assert_eq!(cake::Entity::find_by_name("Cheesecake").one(db).await.unwrap().unwrap().id, 13);
    #[rustfmt::skip]
    assert_eq!(cake::Entity::find_by_name("Chocolate").one(db).await.unwrap().unwrap().id, 15);

    // new load by key feature
    #[rustfmt::skip]
    assert_eq!(cake::Entity::load().filter_by_name("Cheesecake").one(db).await.unwrap().unwrap().id, 13);
    #[rustfmt::skip]
    assert_eq!(cake::Entity::load().filter_by_name("Chocolate").one(db).await.unwrap().unwrap().id, 15);

    let lineitems = lineitem::Entity::load().all(db).await.unwrap();
    assert_eq!(lineitems[0].cake_id, 13);
    assert_eq!(lineitems[1].cake_id, 15);
    assert_eq!(lineitems[0].order_id, 101);
    assert_eq!(lineitems[1].order_id, 101);

    // lineitem join order
    let lineitems = lineitem::Entity::load()
        .with(order::Entity)
        .all(db)
        .await
        .unwrap();
    assert_eq!(lineitems[0].order.as_ref().unwrap().id, 101);
    assert_eq!(lineitems[0].order.as_ref().unwrap().total, 10.into());
    assert_eq!(lineitems[1].order.as_ref().unwrap().id, 101);
    assert_eq!(lineitems[1].order.as_ref().unwrap().total, 10.into());

    // lineitem join cake
    let lineitems = lineitem::Entity::load()
        .with(cake::Entity)
        .all(db)
        .await
        .unwrap();
    assert_eq!(lineitems[0].cake.as_ref().unwrap().id, 13);
    assert_eq!(lineitems[0].cake.as_ref().unwrap().name, "Cheesecake");
    assert_eq!(lineitems[1].cake.as_ref().unwrap().id, 15);
    assert_eq!(lineitems[1].cake.as_ref().unwrap().name, "Chocolate");

    // lineitem join order + cake
    let lineitems = lineitem::Entity::load()
        .with(order::Entity)
        .with(cake::Entity)
        .all(db)
        .await
        .unwrap();
    assert_eq!(lineitems[0].order.as_ref().unwrap().id, 101);
    assert_eq!(lineitems[0].order.as_ref().unwrap().total, 10.into());
    assert_eq!(lineitems[1].order.as_ref().unwrap().id, 101);
    assert_eq!(lineitems[1].order.as_ref().unwrap().total, 10.into());
    assert_eq!(lineitems[0].cake.as_ref().unwrap().id, 13);
    assert_eq!(lineitems[0].cake.as_ref().unwrap().name, "Cheesecake");
    assert_eq!(lineitems[1].cake.as_ref().unwrap().id, 15);
    assert_eq!(lineitems[1].cake.as_ref().unwrap().name, "Chocolate");

    // 1 layer select
    let order = order::Entity::load()
        .with(customer::Entity)
        .order_by_asc(order::Column::Id)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        order,
        order::ModelEx {
            id: 101,
            total: 10.into(),
            bakery_id: 42,
            customer_id: 11,
            placed_at: "2020-01-01 00:00:00Z".parse().unwrap(),
            bakery: HasOne::Unloaded,
            customer: HasOne::loaded(customer::Model {
                id: 11,
                name: "Bob".to_owned(),
                notes: Some("Sweet tooth".into()),
            }),
            lineitems: HasMany::Unloaded,
        }
    );

    // 1 layer select
    let order = order::Entity::load()
        .with(bakery::Entity)
        .with(customer::Entity)
        .order_by_asc(order::Column::Id)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        order,
        order::ModelEx {
            id: 101,
            total: 10.into(),
            bakery_id: 42,
            customer_id: 11,
            placed_at: "2020-01-01 00:00:00Z".parse().unwrap(),
            bakery: HasOne::loaded(bakery::Model {
                id: 42,
                name: "cool little bakery".into(),
                profit_margin: 4.1,
            }),
            customer: HasOne::loaded(customer::ModelEx {
                id: 11,
                name: "Bob".to_owned(),
                notes: Some("Sweet tooth".into()),
                orders: HasMany::Unloaded,
            }),
            lineitems: HasMany::Unloaded,
        }
    );

    // 1 layer select
    let order = order::Entity::load()
        .with(customer::Entity)
        .with(lineitem::Entity)
        .order_by_asc(order::Column::Id)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        order,
        order::ModelEx {
            id: 101,
            total: 10.into(),
            bakery_id: 42,
            customer_id: 11,
            placed_at: "2020-01-01 00:00:00Z".parse().unwrap(),
            bakery: HasOne::Unloaded,
            customer: HasOne::loaded(customer::ModelEx {
                id: 11,
                name: "Bob".to_owned(),
                notes: Some("Sweet tooth".into()),
                orders: HasMany::Unloaded,
            }),
            lineitems: HasMany::Loaded(vec![
                lineitem::ModelEx {
                    id: 1,
                    price: 2.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 13,
                    order: HasOne::Unloaded,
                    cake: HasOne::Unloaded,
                },
                lineitem::ModelEx {
                    id: 2,
                    price: 3.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 15,
                    order: HasOne::Unloaded,
                    cake: HasOne::Unloaded,
                }
            ]),
        }
    );

    // 2 layers
    let order = order::Entity::load()
        .with(customer::Entity)
        .with((lineitem::Entity, cake::Entity))
        .order_by_asc(order::Column::Id)
        .one(db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        order,
        order::ModelEx {
            id: 101,
            total: 10.into(),
            bakery_id: 42,
            customer_id: 11,
            placed_at: "2020-01-01 00:00:00Z".parse().unwrap(),
            bakery: HasOne::Unloaded,
            customer: HasOne::loaded(customer::Model {
                id: 11,
                name: "Bob".to_owned(),
                notes: Some("Sweet tooth".into()),
            }),
            lineitems: HasMany::Loaded(vec![
                lineitem::ModelEx {
                    id: 1,
                    price: 2.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 13,
                    order: HasOne::Unloaded,
                    cake: HasOne::loaded(cake_13),
                },
                lineitem::ModelEx {
                    id: 2,
                    price: 3.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 15,
                    order: HasOne::Unloaded,
                    cake: HasOne::loaded(cake_15),
                }
            ]),
        }
    );
}

#[sea_orm_macros::test]
async fn entity_loader_self_join() -> Result<(), DbErr> {
    use common::film_store::{staff, staff_compact};

    let ctx = TestContext::new("entity_loader_self_join").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(staff::Entity)
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

    let staff = staff::Entity::load()
        .with(staff::Relation::ReportsTo)
        .all(db)
        .await?;

    assert_eq!(staff[0].name, "Alan");
    assert_eq!(staff[0].reports_to, None);

    assert_eq!(staff[1].name, "Ben");
    assert_eq!(staff[1].reports_to.as_ref().unwrap().name, "Alan");

    assert_eq!(staff[2].name, "Alice");
    assert_eq!(staff[2].reports_to.as_ref().unwrap().name, "Alan");

    assert_eq!(staff[3].name, "Elle");
    assert_eq!(staff[3].reports_to, None);

    // test self_ref on compact_model

    let staff = staff_compact::Entity::load()
        .filter_by_id(2)
        .with(staff_compact::Relation::ReportsTo)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(staff.name, "Ben");
    assert_eq!(
        staff.reports_to.unwrap(),
        staff_compact::Entity::find_by_id(alan.id)
            .one(db)
            .await?
            .unwrap()
    );

    // test pagination on loader

    let mut pager = staff::Entity::load()
        .with(staff::Relation::ReportsTo)
        .order_by_asc(staff::COLUMN.id)
        .paginate(db, 2);

    let staff = pager.fetch_and_next().await?.unwrap();

    assert_eq!(staff[0].name, "Alan");
    assert_eq!(staff[0].reports_to, None);

    assert_eq!(staff[1].name, "Ben");
    assert_eq!(staff[1].reports_to.as_ref().unwrap().name, "Alan");

    let staff = pager.fetch_and_next().await?.unwrap();

    assert_eq!(staff[0].name, "Alice");
    assert_eq!(staff[0].reports_to.as_ref().unwrap().name, "Alan");

    assert_eq!(staff[1].name, "Elle");
    assert_eq!(staff[1].reports_to, None);

    assert!(pager.fetch_and_next().await?.is_none());

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
