pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, DatabaseConnection, IntoActiveModel};

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_schema_timestamp_tests").await;

    create_log(&ctx.db).await?;

    ctx.delete().await;

    Ok(())
}

pub async fn create_log(db: &DatabaseConnection) -> Result<(), DbErr> {
    let log = log::Model {
        id: 1,
        json: Json::String("HI".to_owned()),
        created_at: "2021-09-17T17:50:20+08:00".parse().unwrap(),
    };

    let res = Log::insert(log.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(log.id.clone(), res.last_insert_id);
    assert_eq!(Log::find().one(db).await?, Some(log.clone()));

    Ok(())
}
