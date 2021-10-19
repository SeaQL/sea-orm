pub mod common;

pub use common::{features::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("active_enum_tests").await;
    create_tables(&ctx.db).await?;
    insert_active_enum(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_active_enum(db: &DatabaseConnection) -> Result<(), DbErr> {
    active_enum::ActiveModel {
        category: Set(active_enum::Category::Big),
        ..Default::default()
    }
    .insert(db)
    .await?;

    assert_eq!(
        active_enum::Entity::find().one(db).await?.unwrap(),
        active_enum::Model {
            id: 1,
            category: active_enum::Category::Big,
            category_opt: None,
        }
    );

    Ok(())
}
