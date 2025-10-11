#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::create_tables, bakery_dense::*, setup::*};
use sea_orm::{DbConn, DbErr, RuntimeErr, entity::*, query::*};

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
    let baker_3 = insert_baker(db, "Fred", bakery_1.id).await?; // does not make cake

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
    assert!(cakes[0].bakers.get().is_empty());
    assert!(cakes[0].bakery.get().is_none());

    assert_eq!(
        cake::Entity::load()
            .filter(cake::Column::Name.like("Ch%"))
            .order_by_desc(cake::Column::Name)
            .one(db)
            .await?,
        Some(cake_3.clone())
    );

    let mut cakes = cake::Entity::load().with(bakery::Entity).all(db).await?;
    assert_eq!(cakes[0].bakery.get(), Some(&bakery_1));
    assert_eq!(cakes[1].bakery.get(), Some(&bakery_1));
    assert_eq!(cakes[2].bakery.get(), Some(&bakery_2));
    assert_eq!(cakes[3].bakery.get(), None);
    cakes.iter_mut().for_each(|cake| {
        cake.bakery.take();
    });
    assert_eq!(
        cakes,
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone(),
        ]
    );

    let mut cake_with_bakery = cake::Entity::load()
        .filter(cake::Column::Name.eq("Cheesecake"))
        .with(bakery::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(cake_with_bakery.bakery.get(), Some(&bakery_1));
    assert!(cake_with_bakery.bakers.get().is_empty());
    cake_with_bakery.bakery.take();
    cake_with_bakery.bakers.take();
    assert_eq!(cake_with_bakery, cake_1);

    assert_eq!(
        cake::Entity::load()
            .filter_by_id(cake_2.id)
            .with(bakery::Entity)
            .one(db)
            .await?
            .unwrap(),
        {
            let mut cake_2 = cake_2.clone();
            cake_2.bakery.set(Some(bakery_1.clone()));
            cake_2
        }
    );

    let cakes = cake::Entity::load()
        .with(bakery::Entity)
        .with(baker::Entity)
        .all(db)
        .await?;
    assert_eq!(
        cakes
            .iter()
            .cloned()
            .map(|mut cake| {
                cake.bakery.take();
                cake.bakers.take();
                cake
            })
            .collect::<Vec<_>>(),
        [
            cake_1.clone(),
            cake_2.clone(),
            cake_3.clone(),
            cake_4.clone()
        ]
    );
    assert_eq!(cakes[0].bakery.get(), Some(&bakery_1));
    assert_eq!(cakes[1].bakery.get(), Some(&bakery_1));
    assert_eq!(cakes[2].bakery.get(), Some(&bakery_2));
    assert_eq!(cakes[3].bakery.get(), None);
    assert_eq!(cakes[0].bakers.get(), [baker_1.clone()]);
    assert_eq!(cakes[1].bakers.get(), [baker_1.clone(), baker_2.clone()]);
    assert_eq!(cakes[2].bakers.get(), [baker_2.clone()]);
    assert_eq!(cakes[3].bakers.get(), []);

    let mut cake_with_bakery_baker = cake::Entity::load()
        .filter(cake::Column::Name.eq("Chiffon"))
        .with(bakery::Entity)
        .with(baker::Entity)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(cake_with_bakery_baker.bakery.get(), Some(&bakery_2));
    assert_eq!(cake_with_bakery_baker.bakers.get(), [baker_2.clone()]);
    cake_with_bakery_baker.bakery.take();
    cake_with_bakery_baker.bakers.take();
    assert_eq!(cake_with_bakery_baker, cake_3);

    // start again from baker

    let bakers = baker::Entity::find().all(db).await?;
    assert_eq!(bakers, [baker_1.clone(), baker_2.clone(), baker_3.clone()]);

    let bakers = baker::Entity::load().with(cake::Entity).all(db).await?;
    assert_eq!(bakers[0].id, baker_1.id);
    assert_eq!(bakers[1].id, baker_2.id);
    assert_eq!(bakers[2].id, baker_3.id);
    assert_eq!(bakers[0].cakes.get(), [cake_1.clone(), cake_2.clone()]);
    assert_eq!(bakers[1].cakes.get(), [cake_2.clone(), cake_3.clone()]);
    assert_eq!(bakers[2].cakes.get(), []);

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
