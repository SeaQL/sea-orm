pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("json_vec_tests").await;
    create_tables(&ctx.db).await?;
    insert_json_vec(&ctx.db).await?;
    insert_json_vec_derive(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_json_vec(db: &DatabaseConnection) -> Result<(), DbErr> {
    let json_vec = json_vec::Model {
        id: 1,
        str_vec: json_vec::StringVec(vec!["1".to_string(), "2".to_string(), "3".to_string()]),
    };

    let result = json_vec.clone().into_active_model().insert(db).await?;

    assert_eq!(result, json_vec);

    let model = json_vec::Entity::find()
        .filter(json_vec::Column::Id.eq(json_vec.id))
        .one(db)
        .await?;

    assert_eq!(model, Some(json_vec));

    Ok(())
}

pub async fn insert_json_vec_derive(db: &DatabaseConnection) -> Result<(), DbErr> {
    let json_vec = json_vec_derive::Model {
        id: 2,
        str_vec: json_vec_derive::StringVec(vec![
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
        ]),
    };

    let result = json_vec.clone().into_active_model().insert(db).await?;

    assert_eq!(result, json_vec);

    let model = json_vec_derive::Entity::find()
        .filter(json_vec_derive::Column::Id.eq(json_vec.id))
        .one(db)
        .await?;

    assert_eq!(model, Some(json_vec));

    Ok(())
}
