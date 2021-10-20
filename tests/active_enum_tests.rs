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
    use active_enum::*;

    let am = ActiveModel {
        category: Set(None),
        color: Set(None),
        // tea: Set(None),
        ..Default::default()
    }
    .insert(db)
    .await?;

    assert_eq!(
        Entity::find().one(db).await?.unwrap(),
        Model {
            id: 1,
            category: None,
            color: None,
            // tea: None,
        }
    );

    ActiveModel {
        category: Set(Some(Category::Big)),
        color: Set(Some(Color::Black)),
        // tea: Set(Some(Tea::EverydayTea)),
        ..am
    }
    .save(db)
    .await?;

    assert_eq!(
        Entity::find().one(db).await?.unwrap(),
        Model {
            id: 1,
            category: Some(Category::Big),
            color: Some(Color::Black),
            // tea: Some(Tea::EverydayTea),
        }
    );

    Ok(())
}
