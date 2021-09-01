use sea_orm::{entity::prelude::*, DatabaseConnection, Set};
pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};
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

async fn create_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let metadata = metadata::ActiveModel {
        uuid: Set(Uuid::new_v4()),
        key: Set("markup".to_owned()),
        value: Set("1.18".to_owned()),
    };

    let res = Metadata::insert(metadata.clone()).exec(db).await;

    if cfg!(feature = "sqlx-postgres") {
        assert_eq!(metadata.uuid.unwrap(), res?.last_insert_id);
    } else {
        assert_eq!(
            res.unwrap_err(),
            DbErr::Exec("uuid::Uuid cannot be converted from u64".to_owned())
        );
    }

    Ok(())
}
