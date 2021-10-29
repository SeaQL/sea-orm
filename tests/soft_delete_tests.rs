pub mod common;

use chrono::offset::Local;
pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, *};
use sea_query::{ColumnDef, Table};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("soft_delete_tests").await;
    create_tables(&ctx.db).await?;
    test_soft_deletes(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    create_soft_delete_model_table(db).await?;
    create_model_table(db).await?;

    Ok(())
}

pub async fn create_soft_delete_model_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use soft_delete::model_with_soft_delete::*;

    let stmt = Table::create()
        .table(Entity)
        .col(
            ColumnDef::new(Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Column::Name).string().not_null())
        .col(ColumnDef::new(Column::CreatedAt).timestamp())
        .col(ColumnDef::new(Column::UpdatedAt).timestamp())
        .col(ColumnDef::new(Column::DeletedAt).timestamp())
        .to_owned();

    create_table(db, &stmt, Entity).await
}

pub async fn create_model_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use soft_delete::model::*;

    let stmt = Table::create()
        .table(Entity)
        .col(
            ColumnDef::new(Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Column::Name).string().not_null())
        .col(ColumnDef::new(Column::CreatedAt).timestamp())
        .col(ColumnDef::new(Column::UpdatedAt).timestamp())
        .col(ColumnDef::new(Column::DeletedAt).timestamp())
        .to_owned();

    create_table(db, &stmt, Entity).await
}

pub async fn test_soft_deletes(db: &DatabaseConnection) -> Result<(), DbErr> {
    create_and_delete_soft_delete_model(db).await?;
    create_and_delete_model(db).await?;

    Ok(())
}

// Testing model with soft delete enabled
pub async fn create_and_delete_soft_delete_model(db: &DatabaseConnection) -> Result<(), DbErr> {
    use soft_delete::model_with_soft_delete::*;

    // Insert 10 models, and count the number of rows
    for i in 1..=10 {
        ActiveModel {
            name: Set(format!("Model {}", i)),
            ..Default::default()
        }
        .save(db)
        .await?;
    }
    assert_eq!(Entity::find().count(db).await?, 10);

    // Retrieve the first model out
    let model = Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        model,
        Model {
            id: 1,
            name: "Model 1".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Soft delete the retrieved model, and make sure it's soft deleted
    model.into_active_model().delete(db).await?;
    assert_eq!(Entity::find().count(db).await?, 9);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 10);

    // Retrieve the soft deleted model
    let soft_deleted_model = Entity::find_with_deleted()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert!(soft_deleted_model.deleted_at.is_some());

    // Force delete the soft deleted model
    soft_deleted_model
        .into_active_model()
        .delete_forcefully(db)
        .await?;
    assert_eq!(Entity::find().count(db).await?, 9);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 9);

    // Retrieve the second model out
    let model = Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        model,
        Model {
            id: 2,
            name: "Model 2".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Force delete it
    model.into_active_model().delete_forcefully(db).await?;
    assert_eq!(Entity::find().count(db).await?, 8);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 8);

    // Retrieve the third model out
    let model = Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        model,
        Model {
            id: 3,
            name: "Model 3".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Soft delete it
    model.clone().into_active_model().delete(db).await?;
    assert_eq!(Entity::find().count(db).await?, 7);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 8);

    // Revert soft delete
    ActiveModel {
        deleted_at: Set(None),
        ..model.into_active_model()
    }
    .save(db)
    .await?;
    assert_eq!(Entity::find().count(db).await?, 8);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 8);

    Ok(())
}

// Testing model with soft delete disabled
pub async fn create_and_delete_model(db: &DatabaseConnection) -> Result<(), DbErr> {
    use soft_delete::model::*;

    // Insert 10 models, and count the number of rows
    for i in 1..=10 {
        ActiveModel {
            name: Set(format!("Model {}", i)),
            ..Default::default()
        }
        .save(db)
        .await?;
    }
    assert_eq!(Entity::find().count(db).await?, 10);

    // Retrieve the first model out
    let model = Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        model,
        Model {
            id: 1,
            name: "Model 1".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Delete the retrieved model, and make sure it's deleted
    model.into_active_model().delete(db).await?;
    assert_eq!(Entity::find().count(db).await?, 9);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 9);

    // Retrieve the deleted model
    let deleted_model = Entity::find_with_deleted()
        .filter(Column::Id.eq(1))
        .order_by_asc(Column::Id)
        .one(db)
        .await?;
    assert!(deleted_model.is_none());

    // Retrieve the second model out
    let model = Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        model,
        Model {
            id: 2,
            name: "Model 2".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Force delete it
    model.into_active_model().delete_forcefully(db).await?;
    assert_eq!(Entity::find().count(db).await?, 8);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 8);

    // Retrieve the third model out
    let model = Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        model,
        Model {
            id: 3,
            name: "Model 3".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Try to set `deleted_at` with some non-null value
    ActiveModel {
        deleted_at: Set(Some(Local::now().naive_local())),
        ..model.into_active_model()
    }
    .save(db)
    .await?;
    assert_eq!(Entity::find().count(db).await?, 8);
    assert_eq!(Entity::find_with_deleted().count(db).await?, 8);

    // Check it did updated the `deleted_at` column
    assert!(Entity::find()
        .order_by_asc(Column::Id)
        .one(db)
        .await?
        .unwrap()
        .deleted_at
        .is_some());

    Ok(())
}
