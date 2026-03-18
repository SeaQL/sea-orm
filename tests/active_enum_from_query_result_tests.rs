#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult,
    entity::*,
    sea_query::{Expr, ExprTrait, Query},
};

#[sea_orm_macros::test]
async fn from_query_result_with_native_enum() -> Result<(), DbErr> {
    let ctx = TestContext::new("from_query_result_native_enum").await;
    let db = &ctx.db;

    // Setup: create the `tea` enum type and the `active_enum` table
    create_tea_enum(db).await?;
    create_active_enum_table(db).await?;

    // Insert a row with a tea value
    active_enum::ActiveModel {
        id: Set(1),
        category: Set(None),
        color: Set(None),
        tea: Set(Some(Tea::EverydayTea)),
    }
    .insert(db)
    .await?;

    let query = Query::select()
        .column(active_enum::Column::Id)
        .column(active_enum::Column::Tea)
        .from(active_enum::Entity)
        .to_owned();

    #[derive(Debug, PartialEq, FromQueryResult)]
    struct ActiveEnumResult {
        pub id: i32,
        pub tea: Option<Tea>,
    }

    let rows = db.query_all(&query).await?;
    let results: Vec<ActiveEnumResult> = rows
        .iter()
        .map(|r| ActiveEnumResult::from_query_result(r, ""))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        results,
        vec![ActiveEnumResult {
            id: 1,
            tea: Some(Tea::EverydayTea),
        }]
    );

    ctx.delete().await;
    Ok(())
}

/// Verify that explicitly casting the enum column to TEXT still works on Postgres
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
async fn from_query_result_with_cast_works() -> Result<(), DbErr> {
    let ctx = TestContext::new("from_query_result_cast_workaround").await;
    let db = &ctx.db;

    create_tea_enum(db).await?;
    create_active_enum_table(db).await?;

    active_enum::ActiveModel {
        id: Set(1),
        category: Set(None),
        color: Set(None),
        tea: Set(Some(Tea::BreakfastTea)),
    }
    .insert(db)
    .await?;

    // Manually cast enum column to TEXT before reading
    use sea_orm::sea_query::Alias;
    let query = Query::select()
        .column(active_enum::Column::Id)
        .expr_as(
            Expr::col(active_enum::Column::Tea).cast_as(Alias::new("TEXT")),
            Alias::new("tea"),
        )
        .from(active_enum::Entity)
        .to_owned();

    #[derive(Debug, PartialEq, FromQueryResult)]
    struct ActiveEnumResult {
        pub id: i32,
        pub tea: Option<Tea>,
    }

    let rows = db.query_all(&query).await?;
    let results: Vec<ActiveEnumResult> = rows
        .iter()
        .map(|r| ActiveEnumResult::from_query_result(r, ""))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        results,
        vec![ActiveEnumResult {
            id: 1,
            tea: Some(Tea::BreakfastTea),
        }]
    );

    ctx.delete().await;
    Ok(())
}
