pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, DatabaseConnection, IntoActiveModel, Set};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_parallel_tests").await;
    crud_in_parallel(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn crud_in_parallel(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = vec![
        metadata::Model {
            uuid: Uuid::new_v4(),
            key: "markup".to_owned(),
            value: "1.18".to_owned(),
            bytes: vec![1, 2, 3],
            date: Date::from_ymd(2021, 9, 27),
            time: Time::from_hms(11, 32, 55),
        },
        metadata::Model {
            uuid: Uuid::new_v4(),
            key: "exchange_rate".to_owned(),
            value: "0.78".to_owned(),
            bytes: vec![1, 2, 3],
            date: Date::from_ymd(2021, 9, 27),
            time: Time::from_hms(11, 32, 55),
        },
        metadata::Model {
            uuid: Uuid::new_v4(),
            key: "service_charge".to_owned(),
            value: "1.1".to_owned(),
            bytes: vec![1, 2, 3],
            date: Date::from_ymd(2021, 9, 27),
            time: Time::from_hms(11, 32, 55),
        },
    ];

    let _insert_res = futures::try_join!(
        metadata[0].clone().into_active_model().insert(db),
        metadata[1].clone().into_active_model().insert(db),
        metadata[2].clone().into_active_model().insert(db),
    )?;

    let find_res = futures::try_join!(
        Metadata::find_by_id(metadata[0].uuid).one(db),
        Metadata::find_by_id(metadata[1].uuid).one(db),
        Metadata::find_by_id(metadata[2].uuid).one(db),
    )?;

    assert_eq!(
        metadata,
        vec![
            find_res.0.clone().unwrap(),
            find_res.1.clone().unwrap(),
            find_res.2.clone().unwrap(),
        ]
    );

    let mut active_models = (
        find_res.0.unwrap().into_active_model(),
        find_res.1.unwrap().into_active_model(),
        find_res.2.unwrap().into_active_model(),
    );

    active_models.0.bytes = Set(vec![0]);
    active_models.1.bytes = Set(vec![1]);
    active_models.2.bytes = Set(vec![2]);

    let _update_res = futures::try_join!(
        active_models.0.clone().update(db),
        active_models.1.clone().update(db),
        active_models.2.clone().update(db),
    )?;

    let find_res = futures::try_join!(
        Metadata::find_by_id(metadata[0].uuid).one(db),
        Metadata::find_by_id(metadata[1].uuid).one(db),
        Metadata::find_by_id(metadata[2].uuid).one(db),
    )?;

    assert_eq!(
        vec![
            active_models.0.bytes.clone().unwrap(),
            active_models.1.bytes.clone().unwrap(),
            active_models.2.bytes.clone().unwrap(),
        ],
        vec![
            find_res.0.clone().unwrap().bytes,
            find_res.1.clone().unwrap().bytes,
            find_res.2.clone().unwrap().bytes,
        ]
    );

    let _delete_res = futures::try_join!(
        active_models.0.delete(db),
        active_models.1.delete(db),
        active_models.2.delete(db),
    )?;

    assert_eq!(Metadata::find().all(db).await?, vec![]);

    Ok(())
}
