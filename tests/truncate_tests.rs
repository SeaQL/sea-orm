pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::entity::prelude::*;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("truncate_tests").await;
    create_tables(&ctx.db).await?;
    test_truncate(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn test_truncate(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let active_model = ActiveModel {
        ..Default::default()
    };

    active_model.clone().insert(db).await?;
    active_model.clone().insert(db).await?;
    active_model.insert(db).await?;

    assert_eq!(
        Entity::find().all(db).await?,
        [Model { id: 1 }, Model { id: 2 }, Model { id: 3 }]
    );

    assert_eq!(Entity::truncate().exec(db).await?.rows_affected, 3);

    assert_eq!(Entity::find().all(db).await?, []);

    Ok(())
}
