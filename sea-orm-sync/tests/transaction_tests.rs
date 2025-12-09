#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{AccessMode, DatabaseTransaction, IsolationLevel, Set, TransactionTrait, prelude::*};

#[cfg(not(feature = "sync"))]
type FutureResult<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), DbErr>> + Send + 'a>>;
#[cfg(feature = "sync")]
type FutureResult<'a> = Result<(), DbErr>;

fn seaside_bakery() -> bakery::ActiveModel {
    bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
}

fn top_bakery() -> bakery::ActiveModel {
    bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
}

#[sea_orm_macros::test]
pub fn transaction() {
    let ctx = TestContext::new("transaction_test");
    create_tables(&ctx.db).unwrap();

    ctx.db
        .transaction::<_, _, DbErr>(|txn| {
            ({
                let _ = seaside_bakery().save(txn)?;
                let _ = top_bakery().save(txn)?;

                let bakeries = Bakery::find()
                    .filter(bakery::Column::Name.contains("Bakery"))
                    .all(txn)?;

                assert_eq!(bakeries.len(), 2);

                Ok(())
            })
        })
        .unwrap();

    ctx.delete();
}

#[sea_orm_macros::test]
#[cfg(feature = "rbac")]
pub fn rbac_transaction() {
    use sea_orm::rbac::{RbacEngine, RbacSnapshot, RbacUserId};

    let ctx = TestContext::new("rbac_transaction_test");
    create_tables(&ctx.db).unwrap();

    ctx.db.replace_rbac(RbacEngine::from_snapshot(
        RbacSnapshot::danger_unrestricted(),
    ));
    let db = ctx.db.restricted_for(RbacUserId(0)).unwrap();

    db.transaction::<_, _, DbErr>(|txn| {
        ({
            let _ = seaside_bakery().save(txn)?;
            let _ = top_bakery().save(txn)?;

            let bakeries = Bakery::find()
                .filter(bakery::Column::Name.contains("Bakery"))
                .all(txn)?;

            assert_eq!(bakeries.len(), 2);

            Ok(())
        })
    })
    .unwrap();

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn transaction_with_reference() {
    let ctx = TestContext::new("transaction_with_reference_test");
    create_tables(&ctx.db).unwrap();

    let name1 = "SeaSide Bakery";
    let name2 = "Top Bakery";
    let search_name = "Bakery";
    ctx.db
        .transaction(|txn| _transaction_with_reference(txn, name1, name2, search_name))
        .unwrap();

    ctx.delete();
}

fn _transaction_with_reference<'a>(
    txn: &'a DatabaseTransaction,
    name1: &'a str,
    name2: &'a str,
    search_name: &'a str,
) -> FutureResult<'a> {
    ({
        let _ = bakery::ActiveModel {
            name: Set(name1.to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
        .save(txn)?;

        let _ = bakery::ActiveModel {
            name: Set(name2.to_owned()),
            profit_margin: Set(15.0),
            ..Default::default()
        }
        .save(txn)?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains(search_name))
            .all(txn)?;

        assert_eq!(bakeries.len(), 2);

        Ok(())
    })
}

#[sea_orm_macros::test]
pub fn transaction_begin_out_of_scope() -> Result<(), DbErr> {
    let ctx = TestContext::new("transaction_begin_out_of_scope_test");
    create_tables(&ctx.db)?;

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    {
        // Transaction begin in this scope
        let txn = ctx.db.begin()?;

        seaside_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 1);

        top_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 2);

        // The scope ended and transaction is dropped without commit
    }

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
#[cfg(feature = "rbac")]
pub fn rbac_transaction_begin_out_of_scope() -> Result<(), DbErr> {
    use sea_orm::rbac::{RbacEngine, RbacSnapshot, RbacUserId};

    let ctx = TestContext::new("rbac_transaction_begin_out_of_scope_test");
    create_tables(&ctx.db)?;

    ctx.db.replace_rbac(RbacEngine::from_snapshot(
        RbacSnapshot::danger_unrestricted(),
    ));
    let db = ctx.db.restricted_for(RbacUserId(0)).unwrap();

    assert_eq!(bakery::Entity::find().all(&db)?.len(), 0);

    {
        // Transaction begin in this scope
        let txn = db.begin()?;

        seaside_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 1);

        top_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 2);

        // The scope ended and transaction is dropped without commit
    }

    assert_eq!(bakery::Entity::find().all(&db)?.len(), 0);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
pub fn transaction_begin_commit() -> Result<(), DbErr> {
    let ctx = TestContext::new("transaction_begin_commit_test");
    create_tables(&ctx.db)?;

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    {
        // Transaction begin in this scope
        let txn = ctx.db.begin()?;

        seaside_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 1);

        top_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 2);

        // Commit changes before the end of scope
        txn.commit()?;
    }

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 2);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
#[cfg(feature = "rbac")]
pub fn rbac_transaction_begin_commit() -> Result<(), DbErr> {
    use sea_orm::rbac::{RbacEngine, RbacSnapshot, RbacUserId};

    let ctx = TestContext::new("rbac_transaction_begin_commit_test");
    create_tables(&ctx.db)?;

    ctx.db.replace_rbac(RbacEngine::from_snapshot(
        RbacSnapshot::danger_unrestricted(),
    ));
    let db = ctx.db.restricted_for(RbacUserId(0)).unwrap();

    assert_eq!(bakery::Entity::find().all(&db)?.len(), 0);

    {
        // Transaction begin in this scope
        let txn = db.begin()?;

        seaside_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 1);

        top_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 2);

        // Commit changes before the end of scope
        txn.commit()?;
    }

    assert_eq!(bakery::Entity::find().all(&db)?.len(), 2);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
pub fn transaction_begin_rollback() -> Result<(), DbErr> {
    let ctx = TestContext::new("transaction_begin_rollback_test");
    create_tables(&ctx.db)?;

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    {
        // Transaction begin in this scope
        let txn = ctx.db.begin()?;

        seaside_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 1);

        top_bakery().save(&txn)?;

        assert_eq!(bakery::Entity::find().all(&txn)?.len(), 2);

        // Rollback changes before the end of scope
        txn.rollback()?;
    }

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
pub fn transaction_closure_commit() -> Result<(), DbErr> {
    let ctx = TestContext::new("transaction_closure_commit_test");
    create_tables(&ctx.db)?;

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    let res = ctx.db.transaction::<_, _, DbErr>(|txn| {
        ({
            seaside_bakery().save(txn)?;

            assert_eq!(bakery::Entity::find().all(txn)?.len(), 1);

            top_bakery().save(txn)?;

            assert_eq!(bakery::Entity::find().all(txn)?.len(), 2);

            Ok(())
        })
    });

    assert!(res.is_ok());

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 2);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
pub fn transaction_closure_rollback() -> Result<(), DbErr> {
    let ctx = TestContext::new("transaction_closure_rollback_test");
    create_tables(&ctx.db)?;

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    let res = ctx.db.transaction::<_, _, DbErr>(|txn| {
        ({
            seaside_bakery().save(txn)?;

            assert_eq!(bakery::Entity::find().all(txn)?.len(), 1);

            top_bakery().save(txn)?;

            assert_eq!(bakery::Entity::find().all(txn)?.len(), 2);

            bakery::ActiveModel {
                id: Set(1),
                name: Set("Duplicated primary key".to_owned()),
                profit_margin: Set(20.0),
            }
            .insert(txn)?; // Throw error and rollback

            // This line won't be reached
            unreachable!();

            #[allow(unreachable_code)]
            Ok(())
        })
    });

    assert!(res.is_err());

    assert_eq!(bakery::Entity::find().all(&ctx.db)?.len(), 0);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
pub fn transaction_with_active_model_behaviour() -> Result<(), DbErr> {
    let ctx = TestContext::new("transaction_with_active_model_behaviour_test");
    create_tables(&ctx.db)?;

    if let Ok(txn) = ctx.db.begin() {
        assert_eq!(
            cake::ActiveModel {
                name: Set("Cake with invalid price".to_owned()),
                price: Set(rust_dec(0)),
                gluten_free: Set(false),
                ..Default::default()
            }
            .save(&txn),
            Err(DbErr::Custom(
                "[before_save] Invalid Price, insert: true".to_owned()
            ))
        );

        assert_eq!(cake::Entity::find().all(&txn)?.len(), 0);

        assert_eq!(
            cake::ActiveModel {
                name: Set("Cake with invalid price".to_owned()),
                price: Set(rust_dec(-10)),
                gluten_free: Set(false),
                ..Default::default()
            }
            .save(&txn),
            Err(DbErr::Custom(
                "[after_save] Invalid Price, insert: true".to_owned()
            ))
        );

        assert_eq!(cake::Entity::find().all(&txn)?.len(), 1);

        let readonly_cake_1 = cake::ActiveModel {
            name: Set("Readonly cake (err_on_before_delete)".to_owned()),
            price: Set(rust_dec(10)),
            gluten_free: Set(true),
            ..Default::default()
        }
        .save(&txn)?;

        assert_eq!(cake::Entity::find().all(&txn)?.len(), 2);

        assert_eq!(
            readonly_cake_1.delete(&txn).err(),
            Some(DbErr::Custom(
                "[before_delete] Cannot be deleted".to_owned()
            ))
        );

        assert_eq!(cake::Entity::find().all(&txn)?.len(), 2);

        let readonly_cake_2 = cake::ActiveModel {
            name: Set("Readonly cake (err_on_after_delete)".to_owned()),
            price: Set(rust_dec(10)),
            gluten_free: Set(true),
            ..Default::default()
        }
        .save(&txn)?;

        assert_eq!(cake::Entity::find().all(&txn)?.len(), 3);

        assert_eq!(
            readonly_cake_2.delete(&txn).err(),
            Some(DbErr::Custom("[after_delete] Cannot be deleted".to_owned()))
        );

        assert_eq!(cake::Entity::find().all(&txn)?.len(), 2);
    }

    assert_eq!(cake::Entity::find().all(&ctx.db)?.len(), 0);

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
pub fn transaction_nested() {
    let ctx = TestContext::new("transaction_nested_test");
    create_tables(&ctx.db).unwrap();

    ctx.db
        .transaction::<_, _, DbErr>(|txn| {
            ({
                let _ = seaside_bakery().save(txn)?;

                let _ = top_bakery().save(txn)?;

                // Try nested transaction committed
                txn.transaction::<_, _, DbErr>(|txn| {
                    ({
                        let _ = bakery::ActiveModel {
                            name: Set("Nested Bakery".to_owned()),
                            profit_margin: Set(88.88),
                            ..Default::default()
                        }
                        .save(txn)?;

                        let bakeries = Bakery::find()
                            .filter(bakery::Column::Name.contains("Bakery"))
                            .all(txn)?;

                        assert_eq!(bakeries.len(), 3);

                        // Try nested-nested transaction rollbacked
                        let is_err = txn
                            .transaction::<_, _, DbErr>(|txn| {
                                ({
                                    let _ = bakery::ActiveModel {
                                        name: Set("Rock n Roll Bakery".to_owned()),
                                        profit_margin: Set(28.8),
                                        ..Default::default()
                                    }
                                    .save(txn)?;

                                    let bakeries = Bakery::find()
                                        .filter(bakery::Column::Name.contains("Bakery"))
                                        .all(txn)?;

                                    assert_eq!(bakeries.len(), 4);

                                    if true {
                                        Err(DbErr::Query(RuntimeErr::Internal(
                                            "Force Rollback!".to_owned(),
                                        )))
                                    } else {
                                        Ok(())
                                    }
                                })
                            })
                            .is_err();

                        assert!(is_err);

                        let bakeries = Bakery::find()
                            .filter(bakery::Column::Name.contains("Bakery"))
                            .all(txn)?;

                        assert_eq!(bakeries.len(), 3);

                        // Try nested-nested transaction committed
                        txn.transaction::<_, _, DbErr>(|txn| {
                            ({
                                let _ = bakery::ActiveModel {
                                    name: Set("Rock n Roll Bakery".to_owned()),
                                    profit_margin: Set(28.8),
                                    ..Default::default()
                                }
                                .save(txn)?;

                                let bakeries = Bakery::find()
                                    .filter(bakery::Column::Name.contains("Bakery"))
                                    .all(txn)?;

                                assert_eq!(bakeries.len(), 4);

                                Ok(())
                            })
                        })
                        .unwrap();

                        let bakeries = Bakery::find()
                            .filter(bakery::Column::Name.contains("Bakery"))
                            .all(txn)?;

                        assert_eq!(bakeries.len(), 4);

                        Ok(())
                    })
                })
                .unwrap();

                // Try nested transaction rollbacked
                let is_err = txn
                    .transaction::<_, _, DbErr>(|txn| {
                        ({
                            let _ = bakery::ActiveModel {
                                name: Set("Rock n Roll Bakery".to_owned()),
                                profit_margin: Set(28.8),
                                ..Default::default()
                            }
                            .save(txn)?;

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)?;

                            assert_eq!(bakeries.len(), 5);

                            // Try nested-nested transaction committed
                            txn.transaction::<_, _, DbErr>(|txn| {
                                ({
                                    let _ = bakery::ActiveModel {
                                        name: Set("Rock n Roll Bakery".to_owned()),
                                        profit_margin: Set(28.8),
                                        ..Default::default()
                                    }
                                    .save(txn)?;

                                    let bakeries = Bakery::find()
                                        .filter(bakery::Column::Name.contains("Bakery"))
                                        .all(txn)?;

                                    assert_eq!(bakeries.len(), 6);

                                    Ok(())
                                })
                            })
                            .unwrap();

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)?;

                            assert_eq!(bakeries.len(), 6);

                            // Try nested-nested transaction rollbacked
                            let is_err = txn
                                .transaction::<_, _, DbErr>(|txn| {
                                    ({
                                        let _ = bakery::ActiveModel {
                                            name: Set("Rock n Roll Bakery".to_owned()),
                                            profit_margin: Set(28.8),
                                            ..Default::default()
                                        }
                                        .save(txn)?;

                                        let bakeries = Bakery::find()
                                            .filter(bakery::Column::Name.contains("Bakery"))
                                            .all(txn)?;

                                        assert_eq!(bakeries.len(), 7);

                                        if true {
                                            Err(DbErr::Query(RuntimeErr::Internal(
                                                "Force Rollback!".to_owned(),
                                            )))
                                        } else {
                                            Ok(())
                                        }
                                    })
                                })
                                .is_err();

                            assert!(is_err);

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)?;

                            assert_eq!(bakeries.len(), 6);

                            if true {
                                Err(DbErr::Query(RuntimeErr::Internal(
                                    "Force Rollback!".to_owned(),
                                )))
                            } else {
                                Ok(())
                            }
                        })
                    })
                    .is_err();

                assert!(is_err);

                let bakeries = Bakery::find()
                    .filter(bakery::Column::Name.contains("Bakery"))
                    .all(txn)?;

                assert_eq!(bakeries.len(), 4);

                Ok(())
            })
        })
        .unwrap();

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&ctx.db)
        .unwrap();

    assert_eq!(bakeries.len(), 4);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn transaction_manager_nested() -> Result<(), sea_orm::DbErr> {
    let ctx = TestContext::new("transaction_manager_nested");
    create_tables(&ctx.db).unwrap();

    let txn = ctx.db.begin()?;
    let _ = seaside_bakery().save(&txn)?;
    let _ = top_bakery().save(&txn)?;

    // Try nested transaction committed
    {
        let txn = txn.begin()?;
        let _ = bakery::ActiveModel {
            name: Set("Nested Bakery".to_owned()),
            profit_margin: Set(88.88),
            ..Default::default()
        }
        .save(&txn)?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains("Bakery"))
            .all(&txn)?;

        assert_eq!(bakeries.len(), 3);

        // Try nested-nested transaction rollbacked
        {
            let txn = txn.begin()?;
            let _ = bakery::ActiveModel {
                name: Set("Rock n Roll Bakery".to_owned()),
                profit_margin: Set(28.8),
                ..Default::default()
            }
            .save(&txn)?;

            let bakeries = Bakery::find()
                .filter(bakery::Column::Name.contains("Bakery"))
                .all(&txn)?;

            assert_eq!(bakeries.len(), 4);
        }

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains("Bakery"))
            .all(&txn)?;

        assert_eq!(bakeries.len(), 3);

        // Try nested-nested transaction committed
        {
            let txn = txn.begin()?;
            let _ = bakery::ActiveModel {
                name: Set("Rock n Roll Bakery".to_owned()),
                profit_margin: Set(28.8),
                ..Default::default()
            }
            .save(&txn)?;

            let bakeries = Bakery::find()
                .filter(bakery::Column::Name.contains("Bakery"))
                .all(&txn)?;

            assert_eq!(bakeries.len(), 4);
            txn.commit()?;
        }

        txn.commit()?;
    }

    // Try nested transaction rollbacked
    {
        let txn = txn.begin()?;
        let _ = bakery::ActiveModel {
            name: Set("Rock n Roll Bakery".to_owned()),
            profit_margin: Set(28.8),
            ..Default::default()
        }
        .save(&txn)?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains("Bakery"))
            .all(&txn)?;

        assert_eq!(bakeries.len(), 5);

        // Try nested-nested transaction committed
        {
            let txn = txn.begin()?;
            let _ = bakery::ActiveModel {
                name: Set("Rock n Roll Bakery".to_owned()),
                profit_margin: Set(28.8),
                ..Default::default()
            }
            .save(&txn)?;

            let bakeries = Bakery::find()
                .filter(bakery::Column::Name.contains("Bakery"))
                .all(&txn)?;

            assert_eq!(bakeries.len(), 6);
            txn.commit()?;
        }

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains("Bakery"))
            .all(&txn)?;

        assert_eq!(bakeries.len(), 6);
    }

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&txn)?;

    assert_eq!(bakeries.len(), 4);

    txn.commit()?;

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&ctx.db)
        .unwrap();

    assert_eq!(bakeries.len(), 4);

    ctx.delete();

    Ok(())
}

#[sea_orm_macros::test]
#[cfg(feature = "rbac")]
pub fn rbac_transaction_nested() {
    use sea_orm::rbac::{RbacEngine, RbacSnapshot, RbacUserId};

    let ctx = TestContext::new("rbac_transaction_nested_test");
    create_tables(&ctx.db).unwrap();

    ctx.db.replace_rbac(RbacEngine::from_snapshot(
        RbacSnapshot::danger_unrestricted(),
    ));
    let db = ctx.db.restricted_for(RbacUserId(0)).unwrap();

    db.transaction::<_, _, DbErr>(|txn| {
        ({
            let _ = seaside_bakery().save(txn)?;

            let _ = top_bakery().save(txn)?;

            // Try nested transaction committed
            txn.transaction::<_, _, DbErr>(|txn| {
                ({
                    let _ = bakery::ActiveModel {
                        name: Set("Nested Bakery".to_owned()),
                        profit_margin: Set(88.88),
                        ..Default::default()
                    }
                    .save(txn)?;

                    let bakeries = Bakery::find()
                        .filter(bakery::Column::Name.contains("Bakery"))
                        .all(txn)?;

                    assert_eq!(bakeries.len(), 3);

                    // Try nested-nested transaction committed
                    txn.transaction::<_, _, DbErr>(|txn| {
                        ({
                            let _ = bakery::ActiveModel {
                                name: Set("Rock n Roll Bakery".to_owned()),
                                profit_margin: Set(28.8),
                                ..Default::default()
                            }
                            .save(txn)?;

                            let bakeries = Bakery::find()
                                .filter(bakery::Column::Name.contains("Bakery"))
                                .all(txn)?;

                            assert_eq!(bakeries.len(), 4);

                            Ok(())
                        })
                    })
                    .unwrap();

                    let bakeries = Bakery::find()
                        .filter(bakery::Column::Name.contains("Bakery"))
                        .all(txn)?;

                    assert_eq!(bakeries.len(), 4);

                    Ok(())
                })
            })
            .unwrap();

            let bakeries = Bakery::find()
                .filter(bakery::Column::Name.contains("Bakery"))
                .all(txn)?;

            assert_eq!(bakeries.len(), 4);

            Ok(())
        })
    })
    .unwrap();

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&ctx.db)
        .unwrap();

    assert_eq!(bakeries.len(), 4);

    ctx.delete();
}

#[sea_orm_macros::test]
pub fn transaction_with_config() {
    let ctx = TestContext::new("transaction_with_config");
    create_tables(&ctx.db).unwrap();

    for (i, (isolation_level, access_mode)) in [
        (IsolationLevel::RepeatableRead, None),
        (IsolationLevel::ReadCommitted, None),
        (IsolationLevel::ReadUncommitted, Some(AccessMode::ReadWrite)),
        (IsolationLevel::Serializable, Some(AccessMode::ReadWrite)),
    ]
    .into_iter()
    .enumerate()
    {
        let name1 = format!("SeaSide Bakery {}", i);
        let name2 = format!("Top Bakery {}", i);
        let search_name = format!("Bakery {}", i);
        ctx.db
            .transaction_with_config(
                |txn| _transaction_with_config(txn, name1, name2, search_name),
                Some(isolation_level),
                access_mode,
            )
            .unwrap();
    }

    ctx.db
        .transaction_with_config::<_, _, DbErr>(
            |txn| {
                ({
                    let bakeries = Bakery::find()
                        .filter(bakery::Column::Name.contains("Bakery"))
                        .all(txn)?;

                    assert_eq!(bakeries.len(), 8);

                    Ok(())
                })
            },
            None,
            Some(AccessMode::ReadOnly),
        )
        .unwrap();

    ctx.delete();
}

fn _transaction_with_config<'a>(
    txn: &'a DatabaseTransaction,
    name1: String,
    name2: String,
    search_name: String,
) -> FutureResult<'a> {
    ({
        let _ = bakery::ActiveModel {
            name: Set(name1),
            profit_margin: Set(10.4),
            ..Default::default()
        }
        .save(txn)?;

        let _ = bakery::ActiveModel {
            name: Set(name2),
            profit_margin: Set(15.0),
            ..Default::default()
        }
        .save(txn)?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains(&search_name))
            .all(txn)?;

        assert_eq!(bakeries.len(), 2);

        Ok(())
    })
}
