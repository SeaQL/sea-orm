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
    let ctx = TestContext::new("cursor_tests").await;
    create_tables(&ctx.db).await?;
    create_insert_default(&ctx.db).await?;
    cursor_pagination(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_insert_default(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    for _ in 0..10 {
        ActiveModel {
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    assert_eq!(
        Entity::find().all(db).await?,
        vec![
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
            Model { id: 5 },
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    Ok(())
}

pub async fn cursor_pagination(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    // Before 5, i.e. id < 5

    let mut cursor = Entity::find().cursor(Column::Id);

    cursor.before(5);

    assert_eq!(
        cursor.first(4).all(db).await?,
        vec![
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    assert_eq!(
        cursor.first(5).all(db).await?,
        vec![
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    assert_eq!(
        cursor.last(4).all(db).await?,
        vec![
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    assert_eq!(
        cursor.last(5).all(db).await?,
        vec![
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
        ]
    );

    // After 5, i.e. id > 5

    let mut cursor = Entity::find().cursor(Column::Id);

    cursor.after(5);

    assert_eq!(
        cursor.first(4).all(db).await?,
        vec![
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
        ]
    );

    assert_eq!(
        cursor.first(5).all(db).await?,
        vec![
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.first(6).all(db).await?,
        vec![
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.last(4).all(db).await?,
        vec![
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.last(5).all(db).await?,
        vec![
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    assert_eq!(
        cursor.last(6).all(db).await?,
        vec![
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    // Between 5 and 8, i.e. id > 5 AND id < 8

    let mut cursor = Entity::find().cursor(Column::Id);

    cursor.after(5).before(8);

    assert_eq!(cursor.first(1).all(db).await?, vec![Model { id: 6 }]);

    assert_eq!(
        cursor.first(2).all(db).await?,
        vec![Model { id: 6 }, Model { id: 7 }]
    );

    assert_eq!(
        cursor.first(3).all(db).await?,
        vec![Model { id: 6 }, Model { id: 7 }]
    );

    assert_eq!(cursor.last(1).all(db).await?, vec![Model { id: 7 }]);

    assert_eq!(
        cursor.last(2).all(db).await?,
        vec![Model { id: 6 }, Model { id: 7 }]
    );

    assert_eq!(
        cursor.last(3).all(db).await?,
        vec![Model { id: 6 }, Model { id: 7 }]
    );

    Ok(())
}
