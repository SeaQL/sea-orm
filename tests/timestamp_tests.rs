#![allow(unused_imports, dead_code)]

pub mod common;
pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, IntoActiveModel, NotSet, Set, entity::prelude::*};

#[sea_orm_macros::test]
async fn bakery_chain_schema_timestamp_tests() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_schema_timestamp_tests").await;
    create_tables(&ctx.db).await?;
    create_applog(&ctx.db).await?;
    create_satellites_log(&ctx.db).await?;

    ctx.delete().await;

    Ok(())
}

mod access_log {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "access_log")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub ts: ChronoUnixTimestamp,
        pub ms: ChronoUnixTimestampMillis,
        pub tts: TimeUnixTimestamp,
        pub tms: TimeUnixTimestampMillis,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[sea_orm_macros::test]
async fn entity_timestamp_test() -> Result<(), DbErr> {
    let ctx = TestContext::new("entity_timestamp_test").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(access_log::Entity)
        .apply(db)
        .await?;

    let now = sea_orm::prelude::ChronoUtc::now();
    let time_now = sea_orm::prelude::TimeDateTimeWithTimeZone::from_unix_timestamp_nanos(
        now.timestamp_nanos_opt().unwrap() as i128,
    )
    .unwrap();

    let log = access_log::ActiveModel {
        id: NotSet,
        ts: Set(ChronoUnixTimestamp(now)),
        ms: Set(ChronoUnixTimestampMillis(now)),
        tts: Set(TimeUnixTimestamp(time_now)),
        tms: Set(TimeUnixTimestampMillis(time_now)),
    }
    .insert(db)
    .await?;

    assert_eq!(log.ts.timestamp(), now.timestamp());
    assert_eq!(log.ms.timestamp_millis(), now.timestamp_millis());

    assert_eq!(log.tts.unix_timestamp(), now.timestamp());
    assert_eq!(
        log.tms.unix_timestamp_nanos() / 1_000_000,
        now.timestamp_millis() as i128
    );

    #[derive(DerivePartialModel)]
    #[sea_orm(entity = "access_log::Entity")]
    struct AccessLog {
        ts: i64,
        ms: i64,
        tts: i64,
        tms: i64,
    }

    let log: AccessLog = access_log::Entity::find()
        .into_partial_model()
        .one(db)
        .await?
        .unwrap();

    assert_eq!(log.ts, now.timestamp());
    assert_eq!(log.ms, now.timestamp_millis());
    assert_eq!(log.tts, now.timestamp());
    assert_eq!(log.tms, now.timestamp_millis());

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

    assert_eq!(log.id, res.last_insert_id);
    assert_eq!(Applog::find().one(db).await?, Some(log.clone()));

    #[cfg(feature = "sqlx-sqlite")]
    assert_eq!(
        Applog::find().into_json().one(db).await?,
        Some(serde_json::json!({
            "id": 1,
            "action": "Testing",
            "json": r#""HI""#,
            "created_at": "2021-09-17T17:50:20+08:00",
        }))
    );
    #[cfg(feature = "sqlx-mysql")]
    assert_eq!(
        Applog::find().into_json().one(db).await?,
        Some(serde_json::json!({
            "id": 1,
            "action": "Testing",
            "json": "HI",
            "created_at": "2021-09-17T09:50:20Z",
        }))
    );
    #[cfg(feature = "sqlx-postgres")]
    assert_eq!(
        Applog::find().into_json().one(db).await?,
        Some(serde_json::json!({
            "id": 1,
            "action": "Testing",
            "json": "HI",
            "created_at": "2021-09-17T09:50:20Z",
        }))
    );

    Ok(())
}

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

    #[cfg(feature = "sqlx-sqlite")]
    assert_eq!(
        Satellite::find().into_json().one(db).await?,
        Some(serde_json::json!({
            "id": 1,
            "satellite_name": "Sea-00001-2022",
            "launch_date": "2022-01-07T12:11:23+00:00",
            "deployment_date": "2022-01-07T12:11:23Z".parse::<DateTimeLocal>().unwrap().to_rfc3339(),
        }))
    );
    #[cfg(feature = "sqlx-mysql")]
    assert_eq!(
        Satellite::find().into_json().one(db).await?,
        Some(serde_json::json!({
            "id": 1,
            "satellite_name": "Sea-00001-2022",
            "launch_date": "2022-01-07T12:11:23Z",
            "deployment_date": "2022-01-07T12:11:23Z",
        }))
    );
    #[cfg(feature = "sqlx-postgres")]
    assert_eq!(
        Satellite::find().into_json().one(db).await?,
        Some(serde_json::json!({
            "id": 1,
            "satellite_name": "Sea-00001-2022",
            "launch_date": "2022-01-07T12:11:23Z",
            "deployment_date": "2022-01-07T12:11:23Z",
        }))
    );

    Ok(())
}
