pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
pub use sea_orm::{entity::*, query::*, DbConn, DbErr, FromQueryResult};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_one() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_one").await;
    create_tables(&ctx.db).await?;

    let bakery = insert_bakery(&ctx.db, "SeaSide Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Baker 1", bakery.id).await?;

    let baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let bakeries = bakers.load_one(bakery::Entity::find(), &ctx.db).await?;

    assert_eq!(bakers, [baker_1, baker_2]);

    assert_eq!(bakeries, [Some(bakery), None]);

    Ok(())
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_one_complex() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_one_complex").await;
    create_tables(&ctx.db).await?;

    let bakery = insert_bakery(&ctx.db, "SeaSide Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Baker 1", bakery.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Baker 2", bakery.id).await?;

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let bakeries = bakers.load_one(bakery::Entity::find(), &ctx.db).await?;

    assert_eq!(bakers, [baker_1, baker_2]);

    assert_eq!(bakeries, [Some(bakery.clone()), Some(bakery.clone())]);

    Ok(())
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_many() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;
    let bakery_2 = insert_bakery(&ctx.db, "Offshore Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Baker 1", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Baker 2", bakery_1.id).await?;

    let baker_3 = insert_baker(&ctx.db, "John", bakery_2.id).await?;
    let baker_4 = insert_baker(&ctx.db, "Baker 4", bakery_2.id).await?;

    let bakeries = bakery::Entity::find().all(&ctx.db).await?;

    let bakers = bakeries
        .load_many(
            baker::Entity::find().filter(baker::Column::Name.like("Baker%")),
            &ctx.db,
        )
        .await?;

    println!("A: {bakers:?}");
    println!("B: {bakeries:?}");

    assert_eq!(bakeries, [bakery_1.clone(), bakery_2.clone()]);

    assert_eq!(
        bakers,
        [
            vec![baker_1.clone(), baker_2.clone()],
            vec![baker_4.clone()]
        ]
    );

    let bakers = bakeries.load_many(baker::Entity::find(), &ctx.db).await?;

    assert_eq!(
        bakers,
        [
            [baker_1.clone(), baker_2.clone()],
            [baker_3.clone(), baker_4.clone()]
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
            Some(bakery_1),
            Some(bakery_2.clone()),
            Some(bakery_2)
        ]
    );

    Ok(())
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_many_multi() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many_multi").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;
    let bakery_2 = insert_bakery(&ctx.db, "Offshore Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "John", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Jane", bakery_1.id).await?;
    let baker_3 = insert_baker(&ctx.db, "Peter", bakery_2.id).await?;

    let cake_1 = insert_cake(&ctx.db, "Cheesecake", bakery_1.id).await?;
    let cake_2 = insert_cake(&ctx.db, "Chocolate", bakery_2.id).await?;
    let cake_3 = insert_cake(&ctx.db, "Chiffon", bakery_2.id).await?;

    let bakeries = bakery::Entity::find().all(&ctx.db).await?;
    let bakers = bakeries.load_many(baker::Entity::find(), &ctx.db).await?;
    let cakes = bakeries.load_many(cake::Entity::find(), &ctx.db).await?;

    println!("{bakers:?}");
    println!("{bakeries:?}");
    println!("{cakes:?}");

    assert_eq!(bakeries, [bakery_1, bakery_2]);
    assert_eq!(bakers, [vec![baker_1, baker_2], vec![baker_3]]);
    assert_eq!(cakes, [vec![cake_1], vec![cake_2, cake_3]]);

    Ok(())
}

#[ignore]
#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_many_to_many() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_many_to_many").await;
    create_tables(&ctx.db).await?;

    let bakery_1 = insert_bakery(&ctx.db, "SeaSide Bakery").await?;

    let baker_1 = insert_baker(&ctx.db, "Jane", bakery_1.id).await?;
    let baker_2 = insert_baker(&ctx.db, "Peter", bakery_1.id).await?;

    let cake_1 = insert_cake(&ctx.db, "Cheesecake", bakery_1.id).await?;
    let cake_2 = insert_cake(&ctx.db, "Chocolate", bakery_1.id).await?;
    let cake_3 = insert_cake(&ctx.db, "Chiffon", bakery_1.id).await?;

    insert_cake_baker(&ctx.db, baker_1.id, cake_1.id).await?;
    insert_cake_baker(&ctx.db, baker_1.id, cake_2.id).await?;
    insert_cake_baker(&ctx.db, baker_2.id, cake_2.id).await?;
    insert_cake_baker(&ctx.db, baker_2.id, cake_3.id).await?;

    let bakers = baker::Entity::find().all(&ctx.db).await?;
    let cakes = bakers.load_many(cake::Entity::find(), &ctx.db).await?;

    println!("{bakers:?}");
    println!("{cakes:?}");

    assert_eq!(bakers, [baker_1, baker_2]);
    assert_eq!(cakes, [vec![cake_1, cake_2.clone()], vec![cake_2, cake_3]]);

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

pub async fn insert_cake(db: &DbConn, name: &str, bakery_id: i32) -> Result<cake::Model, DbErr> {
    cake::ActiveModel {
        name: Set(name.to_owned()),
        price: Set(rust_decimal::Decimal::ONE),
        gluten_free: Set(false),
        bakery_id: Set(Some(bakery_id)),
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
