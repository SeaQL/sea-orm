pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
pub use sea_orm::{entity::prelude::*, *};
pub use sea_query::Query;

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
        .columns(vec![Column::Name, Column::ProfitMargin])
        .values_panic(vec!["Bakery Shop".into(), 0.5.into()]);

    let mut update = Query::update();
    update
        .table(Entity)
        .values(vec![
            (Column::Name, "Bakery Shop".into()),
            (Column::ProfitMargin, 0.5.into()),
        ])
        .and_where(Column::Id.eq(1));

    let returning =
        Query::returning().columns(vec![Column::Id, Column::Name, Column::ProfitMargin]);

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
