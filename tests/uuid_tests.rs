pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_schema_uuid_tests").await;
    create_and_update_metadata(&ctx.db).await?;
    insert_metadata(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = metadata::Model {
        uuid: Uuid::new_v4(),
        ty: "Type".to_owned(),
        key: "markup".to_owned(),
        value: "1.18".to_owned(),
        bytes: vec![1, 2, 3],
        date: Some(Date::from_ymd(2021, 9, 27)),
        time: Some(Time::from_hms(11, 32, 55)),
    }
    .into_active_model();

    let result = metadata.clone().insert(db).await?;

    assert_eq!(metadata, result);

    Ok(())
}

pub async fn create_and_update_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = metadata::Model {
        uuid: Uuid::new_v4(),
        uuid_ref: None,
        ty: "Type".to_owned(),
        key: "markup".to_owned(),
        value: "1.18".to_owned(),
        bytes: vec![1, 2, 3],
        date: Some(Date::from_ymd(2021, 9, 27)),
        time: Some(Time::from_hms(11, 32, 55)),
    };

    let res = Metadata::insert(metadata.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(Metadata::find().one(db).await?, Some(metadata.clone()));

    assert_eq!(res.last_insert_id, metadata.uuid);

    let update_res = Metadata::update(metadata::ActiveModel {
        value: Set("0.22".to_owned()),
        ..metadata.clone().into_active_model()
    })
    .filter(metadata::Column::Uuid.eq(Uuid::default()))
    .exec(db)
    .await;

    assert_eq!(
        update_res,
        Err(DbErr::RecordNotFound(
            "None of the database rows are affected".to_owned()
        ))
    );

    Ok(())
}
