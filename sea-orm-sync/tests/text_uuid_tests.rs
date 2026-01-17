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
