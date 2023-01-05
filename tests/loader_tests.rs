pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
pub use sea_orm::{entity::*, query::*, DbErr, FromQueryResult};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_one() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_one").await;
    create_tables(&ctx.db).await?;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let baker_1 = baker::ActiveModel {
        name: Set("Baker 1".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(bakery.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let bakers = baker::Entity::find()
        .all(&ctx.db)
        .await
        .expect("Should load bakers");

    let bakeries = bakers
        .load_one(bakery::Entity::find(), &ctx.db)
        .await
        .expect("Should load bakeries");

    assert_eq!(bakers, vec![baker_1, baker_2]);

    assert_eq!(bakeries, vec![Some(bakery), None]);

    Ok(())
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn loader_load_one_complex() -> Result<(), DbErr> {
    let ctx = TestContext::new("loader_test_load_one").await;
    create_tables(&ctx.db).await?;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let baker_1 = baker::ActiveModel {
        name: Set("Baker 1".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(bakery.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(Some(bakery.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let bakers = baker::Entity::find()
        .all(&ctx.db)
        .await
        .expect("Should load bakers");

    let bakeries = bakers
        .load_one(bakery::Entity::find(), &ctx.db)
        .await
        .expect("Should load bakeries");

    assert_eq!(bakers, vec![baker_1, baker_2]);

    assert_eq!(bakeries, vec![Some(bakery.clone()), Some(bakery.clone())]);

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

    let bakery_1 = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let bakery_2 = bakery::ActiveModel {
        name: Set("Offshore Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let baker_1 = baker::ActiveModel {
        name: Set("Baker 1".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(bakery_1.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(Some(bakery_1.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let baker_3 = baker::ActiveModel {
        name: Set("John".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(Some(bakery_2.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let baker_4 = baker::ActiveModel {
        name: Set("Baker 4".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(Some(bakery_2.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let bakeries = bakery::Entity::find()
        .all(&ctx.db)
        .await
        .expect("Should load bakeries");

    let bakers = bakeries
        .load_many(
            baker::Entity::find().filter(baker::Column::Name.like("Baker%")),
            &ctx.db,
        )
        .await
        .expect("Should load bakers");

    println!("A: {:?}", bakers);
    println!("B: {:?}", bakeries);

    assert_eq!(bakeries, vec![bakery_1, bakery_2]);

    assert_eq!(
        bakers,
        vec![
            vec![baker_1.clone(), baker_2.clone()],
            vec![baker_4.clone()]
        ]
    );

    let bakers = bakeries
        .load_many(baker::Entity::find(), &ctx.db)
        .await
        .expect("Should load bakers");

    assert_eq!(bakers, vec![vec![baker_1, baker_2], vec![baker_3, baker_4]]);

    Ok(())
}
