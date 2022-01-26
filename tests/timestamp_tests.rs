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

    Ok(())
}

pub async fn create_applog(db: &DatabaseConnection) -> Result<(), DbErr> {
    let log = applog::Model {
        id: 1,
        action: "Testing".to_owned(),
        json: Json::String("HI".to_owned()),
        jsonb: Json::String("HI".to_owned()),
        created_at: "2021-09-17T17:50:20+08:00".parse().unwrap(),
    };

    let res = Applog::insert(log.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(log.id.clone(), res.last_insert_id);
    assert_eq!(Applog::find().one(db).await?, Some(log.clone()));

    Ok(())
}
