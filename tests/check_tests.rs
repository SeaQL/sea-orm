pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};
use std::{thread, time::Duration};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("check_tests").await;
    create_tables(&ctx.db).await?;
    insert_check(&ctx.db).await?;
    update_check(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_check(db: &DatabaseConnection) -> Result<(), DbErr> {
    use check::*;

    let timestamp = "2022-08-03T00:00:00+08:00"
        .parse::<DateTimeWithTimeZone>()
        .unwrap();

    let model = ActiveModel {
        pay: Set("Billy".to_owned()),
        amount: Set(100.0),
        ..Default::default()
    }
    .insert(db)
    .await?;

    assert_eq!(
        model,
        Entity::find()
            .filter(Column::Id.eq(1))
            .one(db)
            .await?
            .unwrap()
    );

    Check::insert_many([
        ActiveModel {
            pay: Set("Billy".to_owned()),
            amount: Set(100.0),
            ..Default::default()
        },
        ActiveModel {
            pay: Set("Billy".to_owned()),
            amount: Set(100.0),
            created_at: Set(timestamp.clone()),
            ..Default::default()
        },
        ActiveModel {
            pay: Set("Billy".to_owned()),
            amount: Set(100.0),
            updated_at: Set(timestamp.clone()),
            ..Default::default()
        },
        ActiveModel {
            pay: Set("Billy".to_owned()),
            amount: Set(100.0),
            updated_at: Set(timestamp.clone()),
            created_at: Set(timestamp.clone()),
            ..Default::default()
        },
    ])
    .exec(db)
    .await?;

    assert_eq!(5, Entity::find().count(db).await?);

    assert_eq!(
        timestamp,
        Entity::find()
            .filter(Column::Id.eq(3))
            .one(db)
            .await?
            .unwrap()
            .created_at
    );

    assert_eq!(
        timestamp,
        Entity::find()
            .filter(Column::Id.eq(4))
            .one(db)
            .await?
            .unwrap()
            .updated_at
    );

    let model = Entity::find()
        .filter(Column::Id.eq(5))
        .one(db)
        .await?
        .unwrap();

    assert_eq!(timestamp, model.updated_at);
    assert_eq!(timestamp, model.created_at);

    Ok(())
}

pub async fn update_check(db: &DatabaseConnection) -> Result<(), DbErr> {
    use check::*;

    let timestamp = "2022-08-03T16:24:00+08:00"
        .parse::<DateTimeWithTimeZone>()
        .unwrap();

    let model = Entity::find()
        .filter(Column::Id.eq(1))
        .one(db)
        .await?
        .unwrap();

    thread::sleep(Duration::from_secs(1));

    let updated_model = ActiveModel {
        amount: Set(128.0),
        ..model.clone().into_active_model()
    }
    .update(db)
    .await?;

    assert_eq!(128.0, updated_model.amount);
    assert!(model.updated_at < updated_model.updated_at);
    assert!(model.created_at == updated_model.created_at);

    let model = Entity::find()
        .filter(Column::Id.eq(1))
        .one(db)
        .await?
        .unwrap();

    let updated_model = ActiveModel {
        updated_at: Set(timestamp.clone()),
        ..model.clone().into_active_model()
    }
    .update(db)
    .await?;

    assert_eq!(timestamp.clone(), updated_model.updated_at);
    assert!(model.created_at == updated_model.created_at);

    Ok(())
}
