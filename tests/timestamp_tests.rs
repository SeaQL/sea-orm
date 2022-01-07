pub mod common;
pub use common::{features::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, DatabaseConnection, IntoActiveModel};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_schema_timestamp_tests").await;
    create_tables(&ctx.db).await?;
    create_applog(&ctx.db).await?;

    ctx.delete().await;

    {
        let ctx = TestContext::new("bakery_chain_schema_timestamp_tests").await;
        create_tables(&ctx.db).await?;
        create_satellites_log(&ctx.db).await?;

        ctx.delete().await;
    }

    Ok(())
}

pub async fn create_applog(db: &DatabaseConnection) -> Result<(), DbErr> {
    let log = applog::Model {
        id: 1,
        action: "Testing".to_owned(),
        json: Json::String("HI".to_owned()),
        created_at: "2021-09-17T17:50:20+08:00".parse().unwrap(),
    };

    let res = Applog::insert(log.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(log.id.clone(), res.last_insert_id);
    assert_eq!(Applog::find().one(db).await?, Some(log.clone()));

    Ok(())
}

pub async fn create_satellites_log(db: &DatabaseConnection) -> Result<(), DbErr> {
    let archive = datetimeutc::Model {
        id: 1,
        satellite_name: "Sea-00001-2022".to_owned(),
        launch_date: "2022-01-07T12:11:22.500202282Z".parse().unwrap(),
        deployment_date: "2022-01-07T12:11:22.500202282Z".parse().unwrap(),
    };

    let res = DateTimeUtcTest::insert(archive.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(archive.id.clone(), res.last_insert_id);
    assert_eq!(
        DateTimeUtcTest::find().one(db).await?,
        Some(archive.clone())
    );

    Ok(())
}
