#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult,
    entity::*,
    sea_query::{Expr, ExprTrait, Query},
};
use sea_orm::{DeriveActiveEnum, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tea")]
enum TeaString {
    #[sea_orm(string_value = "EverydayTea")]
    EverydayTea,
    #[sea_orm(string_value = "BreakfastTea")]
    BreakfastTea,
    #[sea_orm(string_value = "AfternoonTea")]
    AfternoonTea,
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
fn from_query_result_with_native_pg_enum() -> Result<(), DbErr> {
    let ctx = TestContext::new("from_query_result_native_pg_enum");
    let db = &ctx.db;

    create_tea_enum(db)?;
    create_active_enum_table(db)?;

    active_enum::ActiveModel {
        id: Set(1),
        category: Set(None),
        color: Set(None),
        tea: Set(Some(Tea::EverydayTea)),
    }
    .insert(db)?;

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

    let rows = db.query_all(&query)?;
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

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
fn from_query_result_with_native_pg_enum_rs_type_string() -> Result<(), DbErr> {
    let ctx = TestContext::new("from_query_result_native_pg_enum_rs_type_string");
    let db = &ctx.db;

    create_tea_enum(db)?;
    create_active_enum_table(db)?;

    active_enum::ActiveModel {
        id: Set(1),
        category: Set(None),
        color: Set(None),
        tea: Set(Some(Tea::BreakfastTea)),
    }
    .insert(db)?;

    let query = Query::select()
        .column(active_enum::Column::Id)
        .column(active_enum::Column::Tea)
        .from(active_enum::Entity)
        .to_owned();

    #[derive(Debug, PartialEq, FromQueryResult)]
    struct ActiveEnumStringResult {
        pub id: i32,
        pub tea: Option<TeaString>,
    }

    let rows = db.query_all(&query)?;
    let results: Vec<ActiveEnumStringResult> = rows
        .iter()
        .map(|r| ActiveEnumStringResult::from_query_result(r, ""))
        .collect::<Result<Vec<_>, _>>()?;

    assert_eq!(
        results,
        vec![ActiveEnumStringResult {
            id: 1,
            tea: Some(TeaString::BreakfastTea),
        }]
    );

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
fn from_raw_sql_into_model_with_native_pg_enum() -> Result<(), DbErr> {
    let ctx = TestContext::new("from_raw_sql_native_pg_enum");
    let db = &ctx.db;

    create_tea_enum(db)?;
    create_active_enum_table(db)?;

    active_enum::ActiveModel {
        id: Set(1),
        category: Set(None),
        color: Set(None),
        tea: Set(Some(Tea::EverydayTea)),
    }
    .insert(db)?;

    use sea_orm::{DbBackend, Statement};
    use sea_query::PostgresQueryBuilder;

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

    let stmt = Statement::from_string(DbBackend::Postgres, query.to_string(PostgresQueryBuilder));

    assert_eq!(
        ActiveEnumResult {
            id: 1,
            tea: Some(Tea::EverydayTea),
        },
        active_enum::Entity::find()
            .from_raw_sql(stmt)
            .into_model::<ActiveEnumResult>()
            .one(db)?
            .unwrap()
    );

    ctx.delete();
    Ok(())
}

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
fn from_query_result_with_cast_works() -> Result<(), DbErr> {
    let ctx = TestContext::new("from_query_result_cast_workaround");
    let db = &ctx.db;

    create_tea_enum(db)?;
    create_active_enum_table(db)?;

    active_enum::ActiveModel {
        id: Set(1),
        category: Set(None),
        color: Set(None),
        tea: Set(Some(Tea::BreakfastTea)),
    }
    .insert(db)?;

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

    let rows = db.query_all(&query)?;
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

    ctx.delete();
    Ok(())
}
