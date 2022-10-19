pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("collection_tests").await;
    create_tables(&ctx.db).await?;
    insert_collection(&ctx.db).await?;
    update_collection(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_collection(db: &DatabaseConnection) -> Result<(), DbErr> {
    use collection::*;

    assert_eq!(
        Model {
            id: 1,
            integers: vec![1, 2, 3],
            integers_opt: Some(vec![1, 2, 3]),
        }
        .into_active_model()
        .insert(db)
        .await?,
        Model {
            id: 1,
            integers: vec![1, 2, 3],
            integers_opt: Some(vec![1, 2, 3]),
        }
    );

    assert_eq!(
        Model {
            id: 2,
            integers: vec![10, 9],
            integers_opt: None,
        }
        .into_active_model()
        .insert(db)
        .await?,
        Model {
            id: 2,
            integers: vec![10, 9],
            integers_opt: None,
        }
    );

    assert_eq!(
        Model {
            id: 3,
            integers: vec![],
            integers_opt: Some(vec![]),
        }
        .into_active_model()
        .insert(db)
        .await?,
        Model {
            id: 3,
            integers: vec![],
            integers_opt: Some(vec![]),
        }
    );

    Ok(())
}

pub async fn update_collection(db: &DatabaseConnection) -> Result<(), DbErr> {
    use collection::*;

    let model = Entity::find_by_id(1).one(db).await?.unwrap();

    ActiveModel {
        integers: Set(vec![4, 5, 6]),
        integers_opt: Set(Some(vec![4, 5, 6])),
        ..model.into_active_model()
    }
    .update(db)
    .await?;

    ActiveModel {
        id: Unchanged(3),
        integers: Set(vec![3, 1, 4]),
        integers_opt: Set(None),
    }
    .update(db)
    .await?;

    Ok(())
}
