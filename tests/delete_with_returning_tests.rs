pub mod common;
pub use common::{features::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, DatabaseConnection, IntoActiveModel, Set};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("delete_with_returning").await;
    create_tables(&ctx.db).await?;
    delete_one_with_returning(&ctx.db).await?;
    delete_one_with_returning_not_exist(&ctx.db).await?;

    ctx.delete().await;

    Ok(())
}

pub async fn delete_one_with_returning(db: &DatabaseConnection) -> Result<(), DbErr> {
    let initial_model = applog::Model {
        id: 1,
        action: "Testing".to_owned(),
        json: Json::String("HI".to_owned()),
        created_at: "2021-09-17T17:50:20+08:00".parse().unwrap(),
    };

    Applog::insert(initial_model.clone().into_active_model())
        .exec(db)
        .await?;

    assert_eq!(Applog::find().all(db).await.unwrap().len(), 1);

    let deleted_model = Applog::delete(initial_model.clone().into_active_model())
        .exec_with_returning(db)
        .await?;
    assert_eq!(deleted_model, initial_model);
    assert_eq!(Applog::find().all(db).await.unwrap().len(), 0);
    Ok(())
}

pub async fn delete_one_with_returning_not_exist(db: &DatabaseConnection) -> Result<(), DbErr> {
    let initial_model = applog::Model {
        id: 1,
        action: "Testing".to_owned(),
        json: Json::String("HI".to_owned()),
        created_at: "2021-09-17T17:50:20+08:00".parse().unwrap(),
    };

    Applog::insert(initial_model.into_active_model())
        .exec(db)
        .await?;

    assert_eq!(Applog::find().all(db).await.unwrap().len(), 1);

    let not_exist_model = applog::ActiveModel {
        id: Set(2),
        ..Default::default()
    };

    let deleted_model = Applog::delete(not_exist_model)
        .exec_with_returning(db)
        .await;

    assert_eq!(deleted_model, Err(DbErr::RecordNotUpdated));
    assert_eq!(Applog::find().all(db).await.unwrap().len(), 1);
    Ok(())
}
