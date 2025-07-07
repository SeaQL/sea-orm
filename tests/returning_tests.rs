#![allow(unused_imports, dead_code)]

pub mod common;

use common::{TestContext, bakery_chain, setup::*};
use sea_orm::{IntoActiveModel, NotSet, Set, entity::prelude::*};
use sea_query::{Expr, Query};
use serde_json::json;

#[sea_orm_macros::test]
async fn main() -> Result<(), DbErr> {
    use bakery_chain::bakery::*;

    let ctx = TestContext::new("returning_tests").await;
    let db = &ctx.db;
    let builder = db.get_database_backend();

    let mut insert = Query::insert();
    insert
        .into_table(Entity)
        .columns([Column::Name, Column::ProfitMargin])
        .values_panic(["Bakery Shop".into(), 0.5.into()]);

    let mut update = Query::update();
    update
        .table(Entity)
        .values([
            (Column::Name, "Bakery 2".into()),
            (Column::ProfitMargin, 0.8.into()),
        ])
        .and_where(Column::Id.eq(1));

    let columns = [Column::Id, Column::Name, Column::ProfitMargin];
    let returning =
        Query::returning().exprs(columns.into_iter().map(|c| c.into_returning_expr(builder)));

    bakery_chain::create_tables(db).await?;

    if db.support_returning() {
        insert.returning(returning.clone());
        let insert_res = db
            .query_one(&insert)
            .await?
            .expect("Insert failed with query_one");
        let id: i32 = insert_res.try_get("", "id")?;
        assert_eq!(id, 1);
        let name: String = insert_res.try_get("", "name")?;
        assert_eq!(name, "Bakery Shop");
        let profit_margin: f64 = insert_res.try_get("", "profit_margin")?;
        assert_eq!(profit_margin, 0.5);

        update.returning(returning.clone());
        let update_res = db
            .query_one(&update)
            .await?
            .expect("Update filed with query_one");
        let id: i32 = update_res.try_get("", "id")?;
        assert_eq!(id, 1);
        let name: String = update_res.try_get("", "name")?;
        assert_eq!(name, "Bakery 2");
        let profit_margin: f64 = update_res.try_get("", "profit_margin")?;
        assert_eq!(profit_margin, 0.8);
    } else {
        let insert_res = db.execute(&insert).await?;
        assert!(insert_res.rows_affected() > 0);

        let update_res = db.execute(&update).await?;
        assert!(update_res.rows_affected() > 0);
    }

    ctx.delete().await;

    Ok(())
}

#[sea_orm_macros::test]
async fn insert_many() {
    pub use common::{TestContext, features::*};
    use edit_log::*;

    let ctx = TestContext::new("returning_tests_insert_many").await;
    let db = &ctx.db;

    create_tables(db).await.unwrap();

    Entity::insert(ActiveModel {
        id: NotSet,
        action: Set("one".into()),
        values: Set(json!({ "id": "unique-id-001" })),
    })
    .exec(db)
    .await
    .unwrap();

    assert_eq!(
        Entity::find().all(db).await.unwrap(),
        [Model {
            id: 1,
            action: "one".into(),
            values: json!({ "id": "unique-id-001" }),
        },]
    );

    assert!(
        Entity::insert_many::<ActiveModel, _>([])
            .exec_with_returning(db)
            .await
            .unwrap()
            .is_empty()
    );

    let result = Entity::insert_many([
        ActiveModel {
            id: NotSet,
            action: Set("two".into()),
            values: Set(json!({ "id": "unique-id-002" })),
        },
        ActiveModel {
            id: NotSet,
            action: Set("three".into()),
            values: Set(json!({ "id": "unique-id-003" })),
        },
    ])
    .exec_with_returning(db)
    .await;

    if db.support_returning() {
        assert_eq!(
            result.unwrap(),
            [
                Model {
                    id: 2,
                    action: "two".into(),
                    values: json!({ "id": "unique-id-002" }),
                },
                Model {
                    id: 3,
                    action: "three".into(),
                    values: json!({ "id": "unique-id-003" }),
                },
            ]
        );
    } else {
        assert!(matches!(result, Err(DbErr::BackendNotSupported { .. })));
    }

    assert!(
        Entity::insert_many::<ActiveModel, _>([])
            .exec_with_returning_keys(db)
            .await
            .unwrap()
            .is_empty()
    );

    let result = Entity::insert_many([
        ActiveModel {
            id: NotSet,
            action: Set("four".into()),
            values: Set(json!({ "id": "unique-id-004" })),
        },
        ActiveModel {
            id: NotSet,
            action: Set("five".into()),
            values: Set(json!({ "id": "unique-id-005" })),
        },
    ])
    .exec_with_returning_keys(db)
    .await;

    if db.support_returning() {
        assert_eq!(result.unwrap(), [4, 5]);
    } else {
        assert!(matches!(result, Err(DbErr::BackendNotSupported { .. })));
    }
}

#[sea_orm_macros::test]
async fn insert_many_composite_key() {
    use bakery_chain::{baker, cake, cakes_bakers};
    pub use common::{TestContext, features::*};

    let ctx = TestContext::new("returning_tests_insert_many_composite_key").await;
    let db = &ctx.db;

    bakery_chain::create_tables(db).await.unwrap();

    baker::Entity::insert_many([
        baker::ActiveModel {
            id: NotSet,
            name: Set("Baker 1".to_owned()),
            contact_details: Set(json!(null)),
            bakery_id: Set(None),
        },
        baker::ActiveModel {
            id: NotSet,
            name: Set("Baker 2".to_owned()),
            contact_details: Set(json!(null)),
            bakery_id: Set(None),
        },
    ])
    .exec(db)
    .await
    .unwrap();

    cake::Entity::insert_many([
        cake::ActiveModel {
            id: NotSet,
            name: Set("Cake 1".to_owned()),
            price: Set(Default::default()),
            bakery_id: Set(None),
            gluten_free: Set(false),
            serial: Set(Default::default()),
        },
        cake::ActiveModel {
            id: NotSet,
            name: Set("Cake 2".to_owned()),
            price: Set(Default::default()),
            bakery_id: Set(None),
            gluten_free: Set(false),
            serial: Set(Default::default()),
        },
    ])
    .exec(db)
    .await
    .unwrap();

    let result = cakes_bakers::Entity::insert_many([
        cakes_bakers::ActiveModel {
            cake_id: Set(1),
            baker_id: Set(2),
        },
        cakes_bakers::ActiveModel {
            cake_id: Set(2),
            baker_id: Set(1),
        },
    ])
    .exec_with_returning_keys(db)
    .await;

    if db.support_returning() {
        assert_eq!(result.unwrap(), [(1, 2), (2, 1)]);
    } else {
        assert!(matches!(result, Err(DbErr::BackendNotSupported { .. })));
    }
}

#[sea_orm_macros::test]
async fn update_many() -> Result<(), DbErr> {
    pub use common::{TestContext, features::*};
    use edit_log::*;

    let ctx = TestContext::new("returning_tests_update_many").await;
    let db = &ctx.db;

    create_tables(db).await?;

    Entity::insert(
        Model {
            id: 1,
            action: "before_save".into(),
            values: json!({ "id": "unique-id-001" }),
        }
        .into_active_model(),
    )
    .exec(db)
    .await?;

    Entity::insert(
        Model {
            id: 2,
            action: "before_save".into(),
            values: json!({ "id": "unique-id-002" }),
        }
        .into_active_model(),
    )
    .exec(db)
    .await?;

    Entity::insert(
        Model {
            id: 3,
            action: "before_save".into(),
            values: json!({ "id": "unique-id-003" }),
        }
        .into_active_model(),
    )
    .exec(db)
    .await?;

    assert_eq!(
        Entity::find().all(db).await?,
        [
            Model {
                id: 1,
                action: "before_save".into(),
                values: json!({ "id": "unique-id-001" }),
            },
            Model {
                id: 2,
                action: "before_save".into(),
                values: json!({ "id": "unique-id-002" }),
            },
            Model {
                id: 3,
                action: "before_save".into(),
                values: json!({ "id": "unique-id-003" }),
            },
        ]
    );

    // Update many with returning
    let result = Entity::update_many()
        .col_expr(
            Column::Values,
            Expr::value(json!({ "remarks": "save log" })),
        )
        .filter(Column::Action.eq("before_save"))
        .exec_with_returning(db)
        .await;

    if db.support_returning() {
        assert_eq!(
            result.unwrap(),
            [
                Model {
                    id: 1,
                    action: "before_save".into(),
                    values: json!({ "remarks": "save log" }),
                },
                Model {
                    id: 2,
                    action: "before_save".into(),
                    values: json!({ "remarks": "save log" }),
                },
                Model {
                    id: 3,
                    action: "before_save".into(),
                    values: json!({ "remarks": "save log" }),
                },
            ]
        );
    } else {
        assert!(matches!(result, Err(DbErr::BackendNotSupported { .. })));
    }

    // No-op
    assert_eq!(
        Entity::update_many()
            .filter(Column::Action.eq("before_save"))
            .exec_with_returning(db)
            .await?,
        []
    );

    Ok(())
}

#[sea_orm_macros::test]
async fn delete_many() -> Result<(), DbErr> {
    pub use common::{TestContext, features::*};
    use edit_log::*;

    let ctx = TestContext::new("returning_tests_delete_many").await;
    let db = &ctx.db;

    create_tables(db).await?;

    let inserted_models = [
        Model {
            id: 1,
            action: "before_save".to_string(),
            values: json!({ "id": "unique-id-001" }),
        },
        Model {
            id: 2,
            action: "before_save".to_string(),
            values: json!({ "id": "unique-id-002" }),
        },
    ];

    if db.support_returning() {
        assert_eq!(
            Entity::insert_many(vec![
                ActiveModel {
                    id: NotSet,
                    action: Set("before_save".to_string()),
                    values: Set(json!({ "id": "unique-id-001" })),
                },
                ActiveModel {
                    id: NotSet,
                    action: Set("before_save".to_string()),
                    values: Set(json!({ "id": "unique-id-002" })),
                },
            ])
            .exec_with_returning(db)
            .await
            .unwrap(),
            inserted_models
        );
    } else {
        Entity::insert_many(vec![
            ActiveModel {
                id: NotSet,
                action: Set("before_save".to_string()),
                values: Set(json!({ "id": "unique-id-001" })),
            },
            ActiveModel {
                id: NotSet,
                action: Set("before_save".to_string()),
                values: Set(json!({ "id": "unique-id-002" })),
            },
        ])
        .exec(db)
        .await
        .unwrap();
    }

    if db.support_returning() {
        assert_eq!(
            Entity::delete_many()
                .filter(Column::Action.eq("before_save"))
                .exec_with_returning(db)
                .await
                .unwrap(),
            inserted_models
        );
    } else {
        assert_eq!(
            Entity::delete_many().exec(db).await.unwrap().rows_affected,
            2
        );
    }

    let inserted_model_3 = Model {
        id: 3,
        action: "before_save".to_string(),
        values: json!({ "id": "unique-id-003" }),
    };

    Entity::insert(ActiveModel {
        id: NotSet,
        action: Set("before_save".to_string()),
        values: Set(json!({ "id": "unique-id-003" })),
    })
    .exec(db)
    .await?;

    // Delete one
    if db.support_returning() {
        assert_eq!(
            Entity::delete(ActiveModel {
                id: Set(3),
                ..Default::default()
            })
            .exec_with_returning(db)
            .await
            .unwrap(),
            Some(inserted_model_3)
        );
    } else {
        assert_eq!(
            Entity::delete(ActiveModel {
                id: Set(3),
                ..Default::default()
            })
            .exec(db)
            .await
            .unwrap()
            .rows_affected,
            1
        );
    }

    // No-op
    if db.support_returning() {
        assert_eq!(
            Entity::delete_many().exec_with_returning(db).await.unwrap(),
            []
        );
    } else {
        assert_eq!(
            Entity::delete_many().exec(db).await.unwrap().rows_affected,
            0
        );
    }

    Ok(())
}
