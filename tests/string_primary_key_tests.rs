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
    let ctx = TestContext::new("features_schema_string_primary_key_tests").await;
    create_tables(&ctx.db).await?;
    create_and_update_repository(&ctx.db).await?;
    insert_repository(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_repository(db: &DatabaseConnection) -> Result<(), DbErr> {
    let repository = repository::Model {
        id: "unique-id-001".to_owned(),
        owner: "GC".to_owned(),
        name: "G.C.".to_owned(),
        description: None,
    }
    .into_active_model();

    let result = repository.insert(db).await?;

    assert_eq!(
        result,
        repository::Model {
            id: "unique-id-001".to_owned(),
            owner: "GC".to_owned(),
            name: "G.C.".to_owned(),
            description: None,
        }
    );

    Ok(())
}

pub async fn create_and_update_repository(db: &DatabaseConnection) -> Result<(), DbErr> {
    let repository = repository::Model {
        id: "unique-id-002".to_owned(),
        owner: "GC".to_owned(),
        name: "G.C.".to_owned(),
        description: None,
    };

    let res = Repository::insert(repository.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(Repository::find().one(db).await?, Some(repository.clone()));

    assert_eq!(res.last_insert_id, repository.id);

    let updated_active_model = repository::ActiveModel {
        description: Set(Some("description...".to_owned())),
        ..repository.clone().into_active_model()
    };

    let update_res = Repository::update(updated_active_model.clone())
        .filter(repository::Column::Id.eq("not-exists-id".to_owned()))
        .exec(db)
        .await;

    assert_eq!(
        update_res,
        Err(DbErr::RecordNotFound(
            "None of the database rows are affected".to_owned()
        ))
    );

    let update_res = Repository::update(updated_active_model)
        .filter(repository::Column::Id.eq("unique-id-002".to_owned()))
        .exec(db)
        .await?;

    assert_eq!(
        update_res,
        repository::Model {
            id: "unique-id-002".to_owned(),
            owner: "GC".to_owned(),
            name: "G.C.".to_owned(),
            description: Some("description...".to_owned()),
        }
    );

    let updated_active_model = repository::ActiveModel {
        description: Set(None),
        ..repository.clone().into_active_model()
    };

    let update_res = Repository::update(updated_active_model.clone())
        .filter(repository::Column::Id.eq("unique-id-002".to_owned()))
        .exec(db)
        .await?;

    assert_eq!(
        update_res,
        repository::Model {
            id: "unique-id-002".to_owned(),
            owner: "GC".to_owned(),
            name: "G.C.".to_owned(),
            description: None,
        }
    );

    Ok(())
}
