pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, DatabaseConnection, Set};
use uuid::Uuid;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_schema_primary_key_tests").await;

    create_metadata(&ctx.db).await?;

    ctx.delete().await;

    Ok(())
}

pub async fn create_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = metadata::ActiveModel {
        uuid: Set(Uuid::new_v4()),
        key: Set("markup".to_owned()),
        value: Set("1.18".to_owned()),
    };

    let res = Metadata::insert(metadata.clone()).exec(db).await?;

    assert_eq!(
        res.last_insert_id,
        if cfg!(feature = "sqlx-postgres") {
            metadata.uuid.unwrap()
        } else {
            Default::default()
        }
    );

    Ok(())
}
