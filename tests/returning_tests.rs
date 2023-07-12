pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, IntoActiveModel};
pub use sea_query::{Expr, Query};
use serde_json::json;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    use bakery::*;

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
            (Column::Name, "Bakery Shop".into()),
            (Column::ProfitMargin, 0.5.into()),
        ])
        .and_where(Column::Id.eq(1));

    let returning = Query::returning().columns([Column::Id, Column::Name, Column::ProfitMargin]);

    create_tables(db).await?;

    if db.support_returning() {
        insert.returning(returning.clone());
        let insert_res = db
            .query_one(builder.build(&insert))
            .await?
            .expect("Insert failed with query_one");
        let _id: i32 = insert_res.try_get("", "id")?;
        let _name: String = insert_res.try_get("", "name")?;
        let _profit_margin: f64 = insert_res.try_get("", "profit_margin")?;

        update.returning(returning.clone());
        let update_res = db
            .query_one(builder.build(&update))
            .await?
            .expect("Update filed with query_one");
        let _id: i32 = update_res.try_get("", "id")?;
        let _name: String = update_res.try_get("", "name")?;
        let _profit_margin: f64 = update_res.try_get("", "profit_margin")?;
    } else {
        let insert_res = db.execute(builder.build(&insert)).await?;
        assert!(insert_res.rows_affected() > 0);

        let update_res = db.execute(builder.build(&update)).await?;
        assert!(update_res.rows_affected() > 0);
    }

    ctx.delete().await;

    Ok(())
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg_attr(
    any(feature = "sqlx-mysql", feature = "sqlx-sqlite"),
    should_panic(expected = "Database backend doesn't support RETURNING")
)]
async fn update_many() {
    pub use common::{features::*, setup::*, TestContext};
    use edit_log::*;

    let run = || async {
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
        assert_eq!(
            Entity::update_many()
                .col_expr(
                    Column::Values,
                    Expr::value(json!({ "remarks": "save log" }))
                )
                .filter(Column::Action.eq("before_save"))
                .exec_with_returning(db)
                .await?,
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

        // No-op
        assert_eq!(
            Entity::update_many()
                .filter(Column::Action.eq("before_save"))
                .exec_with_returning(db)
                .await?,
            []
        );

        Result::<(), DbErr>::Ok(())
    };

    run().await.unwrap();
}
