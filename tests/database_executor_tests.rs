#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{prelude::*, DatabaseExecutor, IntoDatabaseExecutor, TransactionTrait};

#[sea_orm_macros::test]
pub async fn connection_or_transaction_from_connection() {
    let ctx = TestContext::new("connection_or_transaction_from_connection").await;
    create_tables(&ctx.db).await.unwrap();

    let cot = DatabaseExecutor::from(&ctx.db);

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&cot)
    .await
    .unwrap();

    let bakeries = Bakery::find().all(&cot).await.unwrap();
    assert_eq!(bakeries.len(), 1);

    ctx.delete().await;
}

#[sea_orm_macros::test]
pub async fn connection_or_transaction_from_transaction() {
    let ctx = TestContext::new("connection_or_transaction_from_transaction").await;
    create_tables(&ctx.db).await.unwrap();

    let txn = ctx.db.begin().await.unwrap();
    let cot = DatabaseExecutor::from(&txn);

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&cot)
    .await
    .unwrap();

    let bakeries = Bakery::find().all(&cot).await.unwrap();
    assert_eq!(bakeries.len(), 1);

    txn.commit().await.unwrap();

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 1);

    ctx.delete().await;
}

#[sea_orm_macros::test]
pub async fn connection_or_transaction_begin() {
    let ctx = TestContext::new("connection_or_transaction_begin").await;
    create_tables(&ctx.db).await.unwrap();

    let cot = DatabaseExecutor::from(&ctx.db);
    let txn = cot.begin().await.unwrap();

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&txn)
    .await
    .unwrap();

    let bakeries = Bakery::find().all(&txn).await.unwrap();
    assert_eq!(bakeries.len(), 1);

    txn.commit().await.unwrap();

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 1);

    ctx.delete().await;
}

#[sea_orm_macros::test]
pub async fn connection_or_transaction_nested() {
    let ctx = TestContext::new("connection_or_transaction_nested").await;
    create_tables(&ctx.db).await.unwrap();

    let txn = ctx.db.begin().await.unwrap();
    let cot = DatabaseExecutor::from(&txn);

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&cot)
    .await
    .unwrap();

    // Begin nested transaction from DatabaseExecutor
    let nested_txn = cot.begin().await.unwrap();

    bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&nested_txn)
    .await
    .unwrap();

    let bakeries = Bakery::find().all(&nested_txn).await.unwrap();
    assert_eq!(bakeries.len(), 2);

    nested_txn.commit().await.unwrap();
    txn.commit().await.unwrap();

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 2);

    ctx.delete().await;
}

#[sea_orm_macros::test]
pub async fn connection_or_transaction_rollback() {
    let ctx = TestContext::new("connection_or_transaction_rollback").await;
    create_tables(&ctx.db).await.unwrap();

    {
        let txn = ctx.db.begin().await.unwrap();
        let cot = DatabaseExecutor::from(&txn);

        bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
        .save(&cot)
        .await
        .unwrap();

        let bakeries = Bakery::find().all(&cot).await.unwrap();
        assert_eq!(bakeries.len(), 1);

        // Transaction dropped without commit - should rollback
    }

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 0);

    ctx.delete().await;
}

#[sea_orm_macros::test]
pub async fn into_database_executor_trait() {
    let ctx = TestContext::new("into_database_executor_trait").await;
    create_tables(&ctx.db).await.unwrap();

    async fn save_bakery<'c, C>(db: C, name: &str) -> Result<(), DbErr>
    where
        C: IntoDatabaseExecutor<'c>,
    {
        let db = db.into_database_executor();
        bakery::ActiveModel {
            name: Set(name.to_owned()),
            profit_margin: Set(10.0),
            ..Default::default()
        }
        .save(&db)
        .await?;
        Ok(())
    }

    // Test with connection
    save_bakery(&ctx.db, "Bakery 1").await.unwrap();

    // Test with transaction
    let txn = ctx.db.begin().await.unwrap();
    save_bakery(&txn, "Bakery 2").await.unwrap();
    txn.commit().await.unwrap();

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 2);

    ctx.delete().await;
}
