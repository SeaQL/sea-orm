pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{DatabaseTransaction, DbErr};
pub use sea_orm::entity::*;
pub use sea_orm::{QueryFilter, ConnectionTrait};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn transaction() {
    let ctx = TestContext::new("transaction_test").await;

    ctx.db.transaction::<_, (), DbErr>(|txn| Box::pin(async move {
        let _ = bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let _ = bakery::ActiveModel {
            name: Set("Top Bakery".to_owned()),
            profit_margin: Set(15.0),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains("Bakery"))
            .all(txn)
            .await?;

        assert_eq!(bakeries.len(), 2);

        Ok(())
    })).await.unwrap();

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn transaction_with_reference() {
    let ctx = TestContext::new("transaction_with_reference_test").await;
    let name1 = "SeaSide Bakery";
    let name2 = "Top Bakery";
    let search_name = "Bakery";
    ctx.db.transaction(|txn| _transaction_with_reference(txn, name1, name2, search_name)).await.unwrap();

    ctx.delete().await;
}

fn _transaction_with_reference<'a>(txn: &'a DatabaseTransaction, name1: &'a str, name2: &'a str, search_name: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(), DbErr>> + Send + 'a>> {
    Box::pin(async move {
        let _ = bakery::ActiveModel {
            name: Set(name1.to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let _ = bakery::ActiveModel {
            name: Set(name2.to_owned()),
            profit_margin: Set(15.0),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains(search_name))
            .all(txn)
            .await?;

        assert_eq!(bakeries.len(), 2);

        Ok(())
    })
}
