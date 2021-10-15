pub mod common;

pub use common::{features::*, setup::*, TestContext};
pub use sea_orm::entity::*;
pub use sea_orm::{ConnectionTrait, QueryFilter};
use sea_orm::{DatabaseTransaction, DbErr};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn transaction() {
    let ctx = TestContext::new("transaction_test").await;
    create_tables(&ctx.db).await;

    ctx.db
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
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
            })
        })
        .await
        .unwrap();

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
    ctx.db
        .transaction(|txn| _transaction_with_reference(txn, name1, name2, search_name))
        .await
        .unwrap();

    ctx.delete().await;
}

fn _transaction_with_reference<'a>(
    txn: &'a DatabaseTransaction,
    name1: &'a str,
    name2: &'a str,
    search_name: &'a str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), DbErr>> + Send + 'a>> {
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

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn transaction_nested() {
    let ctx = TestContext::new("transaction_nested_test").await;

    ctx.db
        .transaction::<_, _, DbErr>(|txn| {
            Box::pin(async move {
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

                // Try nested transaction committed
                txn.transaction::<_, _, DbErr>(|txn| {
                    Box::pin(async move {
                        let _ = bakery::ActiveModel {
                            name: Set("Nested Bakery".to_owned()),
                            profit_margin: Set(88.88),
                            ..Default::default()
                        }
                        .save(txn)
                        .await?;

                        let bakeries = Bakery::find()
                            .filter(bakery::Column::Name.contains("Bakery"))
                            .all(txn)
                            .await?;

                        assert_eq!(bakeries.len(), 3);

                        // Try nested-nested transaction rollbacked
                        let is_err = txn
                            .transaction::<_, _, DbErr>(|txn| {
                                Box::pin(async move {
                                    let _ = bakery::ActiveModel {
                                        name: Set("Rock n Roll Bakery".to_owned()),
                                        profit_margin: Set(28.8),
                                        ..Default::default()
                                    }
                                    .save(txn)
                                    .await?;

                                    let bakeries = Bakery::find()
                                        .filter(bakery::Column::Name.contains("Bakery"))
                                        .all(txn)
                                        .await?;

                                    assert_eq!(bakeries.len(), 4);

                                    if true {
                                        Err(DbErr::Query("Force Rollback!".to_owned()))
                                    } else {
                                        Ok(())
                                    }
                                })
                            })
                            .await
                            .is_err();

                        assert!(is_err);

                        let bakeries = Bakery::find()
                            .filter(bakery::Column::Name.contains("Bakery"))
                            .all(txn)
                            .await?;

                        assert_eq!(bakeries.len(), 3);

                        // Try nested-nested transaction committed
                        txn.transaction::<_, _, DbErr>(|txn| {
                            Box::pin(async move {
                                let _ = bakery::ActiveModel {
                                    name: Set("Rock n Roll Bakery".to_owned()),
                                    profit_margin: Set(28.8),
                                    ..Default::default()
                                }
                                .save(txn)
                                .await?;

                                let bakeries = Bakery::find()
                                    .filter(bakery::Column::Name.contains("Bakery"))
                                    .all(txn)
                                    .await?;

                                assert_eq!(bakeries.len(), 4);

                                Ok(())
                            })
                        })
                        .await
                        .unwrap();

                        let bakeries = Bakery::find()
                            .filter(bakery::Column::Name.contains("Bakery"))
                            .all(txn)
                            .await?;

                        assert_eq!(bakeries.len(), 4);

                        Ok(())
                    })
                })
                .await
                .unwrap();

                // Try nested transaction rollbacked
                let is_err = txn
                    .transaction::<_, _, DbErr>(|txn| {
                        Box::pin(async move {
                            let _ = bakery::ActiveModel {
                                name: Set("Rock n Roll Bakery".to_owned()),
                                profit_margin: Set(28.8),
                                ..Default::default()
                            }
                            .save(txn)
                            .await?;

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)
                                .await?;

                            assert_eq!(bakeries.len(), 5);

                            // Try nested-nested transaction committed
                            txn.transaction::<_, _, DbErr>(|txn| {
                                Box::pin(async move {
                                    let _ = bakery::ActiveModel {
                                        name: Set("Rock n Roll Bakery".to_owned()),
                                        profit_margin: Set(28.8),
                                        ..Default::default()
                                    }
                                    .save(txn)
                                    .await?;

                                    let bakeries = Bakery::find()
                                        .filter(bakery::Column::Name.contains("Bakery"))
                                        .all(txn)
                                        .await?;

                                    assert_eq!(bakeries.len(), 6);

                                    Ok(())
                                })
                            })
                            .await
                            .unwrap();

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)
                                .await?;

                            assert_eq!(bakeries.len(), 6);

                            // Try nested-nested transaction rollbacked
                            let is_err = txn
                                .transaction::<_, _, DbErr>(|txn| {
                                    Box::pin(async move {
                                        let _ = bakery::ActiveModel {
                                            name: Set("Rock n Roll Bakery".to_owned()),
                                            profit_margin: Set(28.8),
                                            ..Default::default()
                                        }
                                        .save(txn)
                                        .await?;

                                        let bakeries = Bakery::find()
                                            .filter(bakery::Column::Name.contains("Bakery"))
                                            .all(txn)
                                            .await?;

                                        assert_eq!(bakeries.len(), 7);

                                        if true {
                                            Err(DbErr::Query("Force Rollback!".to_owned()))
                                        } else {
                                            Ok(())
                                        }
                                    })
                                })
                                .await
                                .is_err();

                            assert!(is_err);

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)
                                .await?;

                            assert_eq!(bakeries.len(), 6);

                            if true {
                                Err(DbErr::Query("Force Rollback!".to_owned()))
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .await
                    .is_err();

                assert!(is_err);

                let bakeries = Bakery::find()
                    .filter(bakery::Column::Name.contains("Bakery"))
                    .all(txn)
                    .await?;

                assert_eq!(bakeries.len(), 4);

                Ok(())
            })
        })
        .await
        .unwrap();

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(bakeries.len(), 4);

    ctx.delete().await;
}
