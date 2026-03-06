#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, FromQueryResult, entity::prelude::*, entity::*};

#[sea_orm_macros::test]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("uuid_fmt_tests").await;
    create_uuid_fmt_table(&ctx.db).await?;
    insert_uuid_fmt(&ctx.db).await?;
    test_text_uuid(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_uuid_fmt(db: &DatabaseConnection) -> Result<(), DbErr> {
    let uuid = Uuid::new_v4();

    let uuid_fmt = uuid_fmt::Model {
        id: 1,
        uuid,
        uuid_braced: uuid.braced(),
        uuid_hyphenated: uuid.hyphenated(),
        uuid_simple: uuid.simple(),
        uuid_urn: uuid.urn(),
    };

    let result = uuid_fmt.clone().into_active_model().insert(db).await?;

    assert_eq!(result, uuid_fmt);

    Ok(())
}

mod uuid_fmt_more {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "uuid_fmt_more")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub iden: TextUuid,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub async fn test_text_uuid(db: &DatabaseConnection) -> Result<(), DbErr> {
    // ensure that column is typed
    let _iden: sea_orm::TextUuidColumn<uuid_fmt_more::Entity> = uuid_fmt_more::COLUMN.iden;

    db.get_schema_builder()
        .register(uuid_fmt_more::Entity)
        .apply(db)
        .await?;

    #[derive(FromQueryResult)]
    struct UuidFmtMore {
        id: i32,
        iden: String, // no casting needed
    }

    let uuid = Uuid::new_v4();
    let model = uuid_fmt_more::ActiveModel {
        iden: Set(uuid.into()),
        ..Default::default()
    };

    let result = model.insert(db).await?;
    assert_eq!(result.iden.0, uuid);

    let result: UuidFmtMore = uuid_fmt_more::Entity::find_by_id(result.id)
        .into_model()
        .one(db)
        .await?
        .unwrap();

    assert_eq!(result.iden, uuid.to_string());

    uuid_fmt_more::ActiveModel {
        iden: Set(Uuid::new_v4().into()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let result = uuid_fmt_more::Entity::find_by_iden(uuid)
        .one(db)
        .await?
        .unwrap();

    assert_eq!(result.iden.0, uuid);

    let result = uuid_fmt_more::Entity::find()
        .filter(uuid_fmt_more::COLUMN.iden.eq(uuid))
        .one(db)
        .await?
        .unwrap();

    assert_eq!(result.iden.0, uuid);

    Ok(())
}
