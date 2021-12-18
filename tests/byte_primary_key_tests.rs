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
    let ctx = TestContext::new("byte_primary_key_tests").await;
    create_tables(&ctx.db).await?;
    create_and_update(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_and_update(db: &DatabaseConnection) -> Result<(), DbErr> {
    use common::features::byte_primary_key::*;

    let model = Model {
        id: vec![1, 2, 3],
        value: "First Row".to_owned(),
    };

    let res = Entity::insert(model.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(Entity::find().one(db).await?, Some(model.clone()));

    assert_eq!(res.last_insert_id, model.id);

    let updated_active_model = ActiveModel {
        value: Set("First Row (Updated)".to_owned()),
        ..model.clone().into_active_model()
    };

    let update_res = Entity::update(updated_active_model.clone())
        .filter(Column::Id.eq(vec![1, 2, 4]))
        .exec(db)
        .await;

    assert_eq!(
        update_res,
        Err(DbErr::RecordNotFound(
            "None of the database rows are affected".to_owned()
        ))
    );

    let update_res = Entity::update(updated_active_model)
        .filter(Column::Id.eq(vec![1, 2, 3]))
        .exec(db)
        .await?;

    assert_eq!(
        update_res,
        Model {
            id: vec![1, 2, 3],
            value: "First Row (Updated)".to_owned(),
        }
    );

    assert_eq!(
        Entity::find()
            .filter(Column::Id.eq(vec![1, 2, 3]))
            .one(db)
            .await?,
        Some(Model {
            id: vec![1, 2, 3],
            value: "First Row (Updated)".to_owned(),
        })
    );

    Ok(())
}
