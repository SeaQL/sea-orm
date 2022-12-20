pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, *};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("self_join_tests").await;
    create_tables(&ctx.db).await?;
    create_metadata(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = self_join::Model {
        uuid: Uuid::new_v4(),
        uuid_ref: None,
        time: Some(Time::from_hms_opt(1, 00, 00).unwrap()),
    };

    model.clone().into_active_model().insert(db).await?;

    let linked_model = self_join::Model {
        uuid: Uuid::new_v4(),
        uuid_ref: Some(model.clone().uuid),
        time: Some(Time::from_hms_opt(2, 00, 00).unwrap()),
    };

    linked_model.clone().into_active_model().insert(db).await?;

    let not_linked_model = self_join::Model {
        uuid: Uuid::new_v4(),
        uuid_ref: None,
        time: Some(Time::from_hms_opt(3, 00, 00).unwrap()),
    };

    not_linked_model
        .clone()
        .into_active_model()
        .insert(db)
        .await?;

    assert_eq!(
        model
            .find_linked(self_join::SelfReferencingLink)
            .all(db)
            .await?,
        vec![]
    );

    assert_eq!(
        linked_model
            .find_linked(self_join::SelfReferencingLink)
            .all(db)
            .await?,
        vec![model.clone()]
    );

    assert_eq!(
        not_linked_model
            .find_linked(self_join::SelfReferencingLink)
            .all(db)
            .await?,
        vec![]
    );

    assert_eq!(
        self_join::Entity::find()
            .find_also_linked(self_join::SelfReferencingLink)
            .order_by_asc(self_join::Column::Time)
            .all(db)
            .await?,
        vec![
            (model.clone(), None),
            (linked_model, Some(model)),
            (not_linked_model, None),
        ]
    );

    Ok(())
}
