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
    let ctx = TestContext::new("uuid_fmt_tests").await;
    create_tables(&ctx.db).await?;
    insert_uuid_fmt(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_uuid_fmt(db: &DatabaseConnection) -> Result<(), DbErr> {
    let uuid = Uuid::new_v4();

    let uuid_fmt = uuid_fmt::Model {
        id: 1,
        uuid: uuid.clone(),
        uuid_braced: uuid.clone().braced(),
        uuid_hyphenated: uuid.clone().hyphenated(),
        uuid_simple: uuid.clone().simple(),
        uuid_urn: uuid.clone().urn(),
    };

    let result = uuid_fmt.clone().into_active_model().insert(db).await?;

    assert_eq!(result, uuid_fmt);

    Ok(())
}
