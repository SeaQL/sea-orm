#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{
    TestContext,
    bakery_chain::{create_tables, seed_data},
    bakery_dense::*,
    setup::*,
};
use sea_orm::{DbConn, DbErr, RuntimeErr, Set, prelude::*, query::*};

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

    let cakes = cake::Entity::load().all(db).await?;
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone(),
        ]
    );

    let cakes = cake::Entity::load()
        .filter(cake::Column::Name.like("Ch%"))
        .all(db)
        .await?;
    assert_eq!(cakes, [cake_1.clone(), cake_3.clone()]);
    assert!(cakes[0].bakers.is_empty());
    assert!(cakes[0].bakery.is_none());

    assert_eq!(
        cake::Entity::load()
            .filter(cake::Column::Name.like("Ch%"))
            .order_by_desc(cake::Column::Name)
            .one(db)
            .await?
            .unwrap(),
        cake_3
    );

    let cakes = cake::Entity::load().with(bakery::Entity).all(db).await?;
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone(),
        ]
    );
    assert_eq!(cakes[0].bakery.as_deref().unwrap(), &bakery_1);
    assert_eq!(cakes[1].bakery.as_deref().unwrap(), &bakery_1);
    assert_eq!(cakes[2].bakery.as_deref().unwrap(), &bakery_2);
    assert_eq!(cakes[3].bakery, None);

    // alternative API
    assert_eq!(
        cakes,
        cake::EntityLoader::load(
            cake::Entity::load().all(db).await?,
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

    let cake_with_bakery = cake::Entity::load()
        .filter(cake::Column::Name.eq("Cheesecake"))
        .with(bakery::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(cake_with_bakery, cake_1);
    assert_eq!(*cake_with_bakery.bakery.unwrap(), bakery_1);
    assert!(cake_with_bakery.bakers.is_empty());

    assert_eq!(
        cake::Entity::load()
            .filter_by_id(cake_2.id)
            .with(bakery::Entity)
            .one(db)
            .await?
            .unwrap(),
        {
            let mut cake_2 = cake_2.clone().into_ex();
            cake_2.bakery = Some(bakery_1.clone().into_ex().into());
            cake_2
        }
    );

    let cakes = cake::Entity::load()
        .with(bakery::Entity)
        .with(baker::Entity)
        .all(db)
        .await?;
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone()
        ]
    );
    assert_eq!(cakes[0].bakery.as_deref().unwrap(), &bakery_1);
    assert_eq!(cakes[1].bakery.as_deref().unwrap(), &bakery_1);
    assert_eq!(cakes[2].bakery.as_deref().unwrap(), &bakery_2);
    assert_eq!(cakes[3].bakery, None);
    assert_eq!(cakes[0].bakers, [baker_1.clone()]);
    assert_eq!(cakes[1].bakers, [baker_1.clone(), baker_2.clone()]);
    assert_eq!(cakes[2].bakers, [baker_2.clone()]);
    assert!(cakes[3].bakers.is_empty());

    let cake_with_bakery_baker = cake::Entity::load()
        .filter(cake::Column::Name.eq("Chiffon"))
        .with(bakery::Entity)
        .with(baker::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(cake_with_bakery_baker, cake_3);
    assert_eq!(*cake_with_bakery_baker.bakery.unwrap(), bakery_2);
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
        .with(baker::Entity)
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

    let cakes = cake::Entity::load()
        .with((bakery::Entity, baker::Entity))
        .all(db)
        .await?;
    assert_eq!(cakes[0].bakery.as_deref().unwrap().name, bakery_1.name);
    assert_eq!(cakes[1].bakery.as_deref().unwrap().name, bakery_1.name);
    assert_eq!(cakes[2].bakery.as_deref().unwrap().name, bakery_2.name);
    assert_eq!(cakes[3].bakery, None);
    assert_eq!(
        cakes[0].bakery.as_deref().unwrap().bakers,
        [baker_1.clone(), baker_2.clone()]
    );
    assert_eq!(
        cakes[1].bakery.as_deref().unwrap().bakers,
        [baker_1.clone(), baker_2.clone()]
    );
    assert_eq!(
        cakes[2].bakery.as_deref().unwrap().bakers,
        [baker_3.clone()]
    );

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
    assert_eq!(lineitems[0].order.as_deref().unwrap().id, 101);
    assert_eq!(lineitems[0].order.as_deref().unwrap().total, 10.into());
    assert_eq!(lineitems[1].order.as_deref().unwrap().id, 101);
    assert_eq!(lineitems[1].order.as_deref().unwrap().total, 10.into());

    // lineitem join cake
    let lineitems = lineitem::Entity::load()
        .with(cake::Entity)
        .all(db)
        .await
        .unwrap();
    assert_eq!(lineitems[0].cake.as_deref().unwrap().id, 13);
    assert_eq!(lineitems[0].cake.as_deref().unwrap().name, "Cheesecake");
    assert_eq!(lineitems[1].cake.as_deref().unwrap().id, 15);
    assert_eq!(lineitems[1].cake.as_deref().unwrap().name, "Chocolate");

    // lineitem join order + cake
    let lineitems = lineitem::Entity::load()
        .with(order::Entity)
        .with(cake::Entity)
        .all(db)
        .await
        .unwrap();
    assert_eq!(lineitems[0].order.as_deref().unwrap().id, 101);
    assert_eq!(lineitems[0].order.as_deref().unwrap().total, 10.into());
    assert_eq!(lineitems[1].order.as_deref().unwrap().id, 101);
    assert_eq!(lineitems[1].order.as_deref().unwrap().total, 10.into());
    assert_eq!(lineitems[0].cake.as_deref().unwrap().id, 13);
    assert_eq!(lineitems[0].cake.as_deref().unwrap().name, "Cheesecake");
    assert_eq!(lineitems[1].cake.as_deref().unwrap().id, 15);
    assert_eq!(lineitems[1].cake.as_deref().unwrap().name, "Chocolate");

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
            bakery: None,
            customer: Some(
                customer::ModelEx {
                    id: 11,
                    name: "Bob".to_owned(),
                    notes: Some("Sweet tooth".into()),
                    orders: vec![],
                }
                .into()
            ),
            lineitems: vec![],
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
            bakery: Some(
                bakery::ModelEx {
                    id: 42,
                    name: "cool little bakery".into(),
                    profit_margin: 4.1,
                    bakers: vec![],
                    cakes: vec![],
                    orders: vec![],
                }
                .into()
            ),
            customer: Some(
                customer::ModelEx {
                    id: 11,
                    name: "Bob".to_owned(),
                    notes: Some("Sweet tooth".into()),
                    orders: vec![],
                }
                .into()
            ),
            lineitems: vec![],
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
            bakery: None,
            customer: Some(
                customer::ModelEx {
                    id: 11,
                    name: "Bob".to_owned(),
                    notes: Some("Sweet tooth".into()),
                    orders: vec![],
                }
                .into()
            ),
            lineitems: vec![
                lineitem::ModelEx {
                    id: 1,
                    price: 2.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 13,
                    order: None,
                    cake: None,
                },
                lineitem::ModelEx {
                    id: 2,
                    price: 3.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 15,
                    order: None,
                    cake: None,
                }
            ],
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
            bakery: None,
            customer: Some(
                customer::ModelEx {
                    id: 11,
                    name: "Bob".to_owned(),
                    notes: Some("Sweet tooth".into()),
                    orders: vec![],
                }
                .into()
            ),
            lineitems: vec![
                lineitem::ModelEx {
                    id: 1,
                    price: 2.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 13,
                    order: None,
                    cake: Some(cake_13.into_ex().into()),
                },
                lineitem::ModelEx {
                    id: 2,
                    price: 3.into(),
                    quantity: 2,
                    order_id: 101,
                    cake_id: 15,
                    order: None,
                    cake: Some(cake_15.into_ex().into()),
                }
            ],
        }
    );
}

#[sea_orm_macros::test]
async fn entity_loader_exp() -> Result<(), DbErr> {
    let ctx = TestContext::new("entity_loader_exp").await;
    create_tables(&ctx.db).await.unwrap();
    seed_data::init_1(&ctx, true).await;
    let db = &ctx.db;

    let loader = lineitem::Entity::load()
        .with(cake::Entity)
        .with(order::Entity);
    println!("{loader:?}");
    loader.all(db).await?;

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
