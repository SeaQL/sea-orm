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
    create_satellites_log(&ctx.db).await?;

    ctx.delete().await;

    Ok(())
}

#[cfg(feature = "with-chrono")]
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

    assert_eq!(log.id, res.last_insert_id);
    assert_eq!(Applog::find().one(db).await?, Some(log.clone()));

    Ok(())
}

#[cfg(feature = "with-time")]
pub async fn create_applog(db: &DatabaseConnection) -> Result<(), DbErr> {
    let log = applog::Model {
        id: 1,
        action: "Testing".to_owned(),
        json: Json::String("HI".to_owned()),
        created_at: time::OffsetDateTime::parse("2021-09-17T17:50:20+08:00", time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]")).unwrap(),
    };

    let res = Applog::insert(log.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(log.id, res.last_insert_id);
    assert_eq!(Applog::find().one(db).await?, Some(log.clone()));

    Ok(())
}

#[cfg(feature = "with-chrono")]
pub async fn create_satellites_log(db: &DatabaseConnection) -> Result<(), DbErr> {
    let archive = satellite::Model {
        id: 1,
        satellite_name: "Sea-00001-2022".to_owned(),
        launch_date: "2022-01-07T12:11:23Z".parse().unwrap(),
        deployment_date: "2022-01-07T12:11:23Z".parse().unwrap(),
    };

    let res = Satellite::insert(archive.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(archive.id, res.last_insert_id);
    assert_eq!(Satellite::find().one(db).await?, Some(archive.clone()));

    Ok(())
}

#[cfg(feature = "with-time")]
pub async fn create_satellites_log(db: &DatabaseConnection) -> Result<(), DbErr> {
    let archive = satellite::Model {
        id: 1,
        satellite_name: "Sea-00001-2022".to_owned(),
        launch_date: time::OffsetDateTime::parse(
            "2022-01-07T12:11:23Z",
            time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z"),
        )
        .unwrap(),
        deployment_date: time::OffsetDateTime::parse(
            "2022-01-07T12:11:23Z",
            time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z"),
        )
        .unwrap(),
    };

    let res = Satellite::insert(archive.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(archive.id, res.last_insert_id);
    assert_eq!(Satellite::find().one(db).await?, Some(archive.clone()));

    Ok(())
}
