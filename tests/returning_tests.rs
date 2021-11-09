pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{entity::prelude::*, *};
use sea_query::Query;

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
    
    let mut returning = Query::select();
    returning.columns(vec![Column::Id, Column::Name, Column::ProfitMargin]);
    if db.returning_on_insert() {
        insert.returning(returning.clone());
    }
    if db.returning_on_update() {
        update.returning(returning.clone());
    }

    create_tables(db).await?;
    println!("db_version: {:#?}", db.version());
    db.query_one(builder.build(&insert)).await?;
    db.query_one(builder.build(&update)).await?;
    ctx.delete().await;

    Ok(())
}
