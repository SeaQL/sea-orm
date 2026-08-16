pub mod common;
use common::{TestContext, features::*, setup::*};
use sea_orm::{DatabaseConnection, IntoActiveModel, NotSet, Set, entity::prelude::*};
use uuid::Uuid;

mod sample {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sample")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: TextUuid,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[sea_orm_macros::test]
fn text_uuid_test() -> Result<(), DbErr> {
    let ctx = TestContext::new("text_uuid_test");
    let db = &ctx.db;

    let uuid = Uuid::new_v4();

    db.get_schema_builder().register(sample::Entity).apply(db)?;

    let entry = sample::ActiveModel {
        id: Set(uuid.into()),
    }
    .insert(db)?;

    assert_eq!(*entry.id, uuid);

    Ok(())
}

#[cfg(all(feature = "serde", feature = "with-json"))]
#[test]
fn text_uuid_serde_uses_uuid_string() -> Result<(), serde_json::Error> {
    let uuid = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
    let text_uuid = TextUuid::from(uuid);

    let json = serde_json::to_string(&text_uuid)?;
    assert_eq!(json, format!("\"{uuid}\""));
    assert_eq!(serde_json::from_str::<TextUuid>(&json)?, text_uuid);
    assert!(serde_json::from_str::<TextUuid>("\"not-a-uuid\"").is_err());

    Ok(())
}
