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

    let model = Model {
        id: 1,
        category: None,
        color: None,
        tea: None,
    };

    assert_eq!(
        model,
        ActiveModel {
            category: Set(None),
            color: Set(None),
            tea: Set(None),
            ..Default::default()
        }
        .insert(db)
        .await?
    );
    assert_eq!(model, Entity::find().one(db).await?.unwrap());
    assert_eq!(
        model,
        Entity::find()
            .filter(Column::Id.is_not_null())
            .filter(Column::Category.is_null())
            .filter(Column::Color.is_null())
            .filter(Column::Tea.is_null())
            .one(db)
            .await?
            .unwrap()
    );

    let _ = ActiveModel {
        category: Set(Some(Category::Big)),
        color: Set(Some(Color::Black)),
        tea: Set(Some(Tea::EverydayTea)),
        ..model.into_active_model()
    }
    .save(db)
    .await?;

    let model = Entity::find().one(db).await?.unwrap();
    assert_eq!(
        model,
        Model {
            id: 1,
            category: Some(Category::Big),
            color: Some(Color::Black),
            tea: Some(Tea::EverydayTea),
        }
    );
    assert_eq!(
        model,
        Entity::find()
            .filter(Column::Id.eq(1))
            .filter(Column::Category.eq(Category::Big))
            .filter(Column::Color.eq(Color::Black))
            .filter(Column::Tea.eq(Tea::EverydayTea))
            .one(db)
            .await?
            .unwrap()
    );

    let res = model.into_active_model().delete(db).await?;

    assert_eq!(res.rows_affected, 1);
    assert_eq!(Entity::find().one(db).await?, None);

    Ok(())
}
