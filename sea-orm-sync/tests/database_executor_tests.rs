#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseExecutor, IntoDatabaseExecutor, Set, TransactionTrait, prelude::*};

#[sea_orm_macros::test]
pub fn connection_or_transaction_from_connection() {
    let ctx = TestContext::new("connection_or_transaction_from_connection");
    create_tables(&ctx.db).unwrap();

    let cot = DatabaseExecutor::from(&ctx.db);

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&cot)
    .unwrap();

    let bakeries = Bakery::find().all(&cot).unwrap();
    assert_eq!(bakeries.len(), 1);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn connection_or_transaction_from_transaction() {
    let ctx = TestContext::new("connection_or_transaction_from_transaction");
    create_tables(&ctx.db).unwrap();

    let txn = ctx.db.begin().unwrap();
    let cot = DatabaseExecutor::from(&txn);

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&cot)
    .unwrap();

    let bakeries = Bakery::find().all(&cot).unwrap();
    assert_eq!(bakeries.len(), 1);

    txn.commit().unwrap();

    let bakeries = Bakery::find().all(&ctx.db).unwrap();
    assert_eq!(bakeries.len(), 1);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn connection_or_transaction_begin() {
    let ctx = TestContext::new("connection_or_transaction_begin");
    create_tables(&ctx.db).unwrap();

    let cot = DatabaseExecutor::from(&ctx.db);
    let txn = cot.begin().unwrap();

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&txn)
    .unwrap();

    let bakeries = Bakery::find().all(&txn).unwrap();
    assert_eq!(bakeries.len(), 1);

    txn.commit().unwrap();

    let bakeries = Bakery::find().all(&ctx.db).unwrap();
    assert_eq!(bakeries.len(), 1);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn connection_or_transaction_nested() {
    let ctx = TestContext::new("connection_or_transaction_nested");
    create_tables(&ctx.db).unwrap();

    let txn = ctx.db.begin().unwrap();
    let cot = DatabaseExecutor::from(&txn);

    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&cot)
    .unwrap();

    // Begin nested transaction from DatabaseExecutor
    let nested_txn = cot.begin().unwrap();

    bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&nested_txn)
    .unwrap();

    let bakeries = Bakery::find().all(&nested_txn).unwrap();
    assert_eq!(bakeries.len(), 2);

    nested_txn.commit().unwrap();
    txn.commit().unwrap();

    let bakeries = Bakery::find().all(&ctx.db).unwrap();
    assert_eq!(bakeries.len(), 2);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn connection_or_transaction_rollback() {
    let ctx = TestContext::new("connection_or_transaction_rollback");
    create_tables(&ctx.db).unwrap();

    {
        let txn = ctx.db.begin().unwrap();
        let cot = DatabaseExecutor::from(&txn);

        bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
        .save(&cot)
        .unwrap();

        let bakeries = Bakery::find().all(&cot).unwrap();
        assert_eq!(bakeries.len(), 1);

        // Transaction dropped without commit - should rollback
    }

    let bakeries = Bakery::find().all(&ctx.db).unwrap();
    assert_eq!(bakeries.len(), 0);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn into_database_executor_trait() {
    let ctx = TestContext::new("into_database_executor_trait");
    create_tables(&ctx.db).unwrap();

    fn save_bakery<'c, C>(db: C, name: &str) -> Result<(), DbErr>
    where
        C: IntoDatabaseExecutor<'c>,
    {
        let db = db.into_database_executor();
        bakery::ActiveModel {
            name: Set(name.to_owned()),
            profit_margin: Set(10.0),
            ..Default::default()
        }
        .save(&db)?;
        Ok(())
    }

    // Test with connection
    save_bakery(&ctx.db, "Bakery 1").unwrap();

    // Test with transaction
    let txn = ctx.db.begin().unwrap();
    save_bakery(&txn, "Bakery 2").unwrap();
    txn.commit().unwrap();

    let bakeries = Bakery::find().all(&ctx.db).unwrap();
    assert_eq!(bakeries.len(), 2);

    ctx.delete();
}
