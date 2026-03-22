#![allow(unused_imports, dead_code)]

pub mod common;

use crate::common::TestContext;
use sea_orm::{
    DatabaseBackend, DatabaseConnection, DbErr, Statement,
    entity::*,
    query::*,
    sea_query::{Condition, Expr, Query},
};

// Scenario 1: table is first synced with a `#[sea_orm(unique)]` column already
// present. Repeated syncs must not drop the column-level unique constraint.
mod item_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_item")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Scenario 2a: initial version of the table — no unique column yet.
mod product_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_product")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Scenario 2b: updated version — a unique column is added.
mod product_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_product")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub sku: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Scenario 4a: initial version — column has UNIQUE.
mod order_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_order")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub ref_no: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Scenario 4b: UNIQUE removed from the column.
mod order_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_order")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub ref_no: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Scenario 3a: initial version — column exists without UNIQUE.
mod user_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub email: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Scenario 3b: updated version — the existing column is made unique.
mod user_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub email: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// Entity in a non-default PostgreSQL schema — for multi-schema sync testing.
mod custom_schema_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "test_schema_2952", table_name = "sync_custom_schema")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Regression test for <https://github.com/SeaQL/sea-orm/issues/2970>.
///
/// A table with a `#[sea_orm(unique)]` column is created on the first sync.
/// The subsequent sync must not attempt to drop the column-level unique index.
#[sea_orm_macros::test]
async fn test_sync_unique_column_no_drop() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_unique_column_no_drop").await;
    let db = &ctx.db;

    #[cfg(feature = "schema-sync")]
    {
        // First sync: creates the table with the unique column
        db.get_schema_builder()
            .register(item_v1::Entity)
            .sync(db)
            .await?;

        // Second sync: must not try to drop the column-level unique index
        db.get_schema_builder()
            .register(item_v1::Entity)
            .sync(db)
            .await?;

        #[cfg(feature = "sqlx-postgres")]
        assert!(
            pg_index_exists(db, "sync_item", "sync_item_name_key").await?,
            "unique index on `sync_item.name` should still exist after repeated sync"
        );
    }

    Ok(())
}

/// Regression test for <https://github.com/SeaQL/sea-orm/issues/2970>.
///
/// A unique column is added to an existing table via sync (ALTER TABLE ADD
/// COLUMN … UNIQUE), which creates a column-level unique index. A subsequent
/// sync must not attempt to drop that index.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
async fn test_sync_add_unique_column_no_drop() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_add_unique_column_no_drop").await;
    let db = &ctx.db;

    #[cfg(feature = "schema-sync")]
    {
        // First sync: creates the table without the unique column
        db.get_schema_builder()
            .register(product_v1::Entity)
            .sync(db)
            .await?;

        // Second sync: adds the unique column via ALTER TABLE ADD COLUMN … UNIQUE
        db.get_schema_builder()
            .register(product_v2::Entity)
            .sync(db)
            .await?;

        // Third sync: must not try to drop the unique index created above
        db.get_schema_builder()
            .register(product_v2::Entity)
            .sync(db)
            .await?;

        #[cfg(feature = "sqlx-postgres")]
        assert!(
            pg_index_exists(db, "sync_product", "sync_product_sku_key").await?,
            "unique index on `sync_product.sku` should still exist after repeated sync"
        );
    }

    Ok(())
}

/// Scenario 3: an existing column is made unique in a later sync.
///
/// When a column that already exists in the DB is annotated with
/// `#[sea_orm(unique)]`, the sync must create a unique index for it.
#[sea_orm_macros::test]
async fn test_sync_make_existing_column_unique() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_make_existing_column_unique").await;
    let db = &ctx.db;

    #[cfg(feature = "schema-sync")]
    {
        // First sync: creates the table with a plain (non-unique) email column
        db.get_schema_builder()
            .register(user_v1::Entity)
            .sync(db)
            .await?;

        // Second sync: email is now marked unique — should create the unique index
        db.get_schema_builder()
            .register(user_v2::Entity)
            .sync(db)
            .await?;

        // Third sync: must not try to drop or re-create the index
        db.get_schema_builder()
            .register(user_v2::Entity)
            .sync(db)
            .await?;

        #[cfg(feature = "sqlx-postgres")]
        assert!(
            pg_index_exists(db, "sync_user", "idx-sync_user-email").await?,
            "unique index on `sync_user.email` should be created when column is made unique"
        );
    }

    Ok(())
}

/// Regression test for <https://github.com/SeaQL/sea-orm/issues/2994>.
///
/// A column marked `#[sea_orm(unique)]` is synced, then the unique attribute is
/// removed. The second sync must drop the PostgreSQL constraint without error.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_sync_drop_unique_constraint() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_drop_unique_constraint").await;
    let db = &ctx.db;

    #[cfg(feature = "schema-sync")]
    {
        // First sync: creates the table with the unique constraint
        db.get_schema_builder()
            .register(order_v1::Entity)
            .sync(db)
            .await?;

        assert!(
            pg_index_exists(db, "sync_order", "sync_order_ref_no_key").await?,
            "unique constraint should exist after first sync"
        );

        // Second sync: unique is removed — must not error on PostgreSQL
        db.get_schema_builder()
            .register(order_v2::Entity)
            .sync(db)
            .await?;

        assert!(
            !pg_index_exists(db, "sync_order", "sync_order_ref_no_key").await?,
            "unique constraint should be gone after second sync"
        );
    }

    Ok(())
}

/// Regression test for <https://github.com/SeaQL/sea-orm/issues/2952>.
///
/// An entity with `schema_name` pointing to a non-default PostgreSQL schema is
/// synced twice. The second sync must not fail with "relation already exists".
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_sync_non_default_schema() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_non_default_schema").await;
    let db = &ctx.db;

    #[cfg(feature = "schema-sync")]
    {
        db.execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "CREATE SCHEMA IF NOT EXISTS test_schema_2952".to_owned(),
        ))
        .await?;

        // First sync: creates the table in the non-default schema
        db.get_schema_builder()
            .register(custom_schema_entity::Entity)
            .sync(db)
            .await?;

        assert!(
            pg_table_exists_in_schema(db, "test_schema_2952", "sync_custom_schema").await?,
            "table should exist in schema `test_schema_2952`"
        );

        assert!(
            !pg_table_exists_in_schema(db, "public", "sync_custom_schema").await?,
            "table should NOT exist in schema `public`"
        );

        // Second sync: must not fail with "relation already exists"
        db.get_schema_builder()
            .register(custom_schema_entity::Entity)
            .sync(db)
            .await?;
    }

    Ok(())
}

#[cfg(feature = "sqlx-postgres")]
async fn pg_table_exists_in_schema(
    db: &DatabaseConnection,
    schema: &str,
    table: &str,
) -> Result<bool, DbErr> {
    db.query_one(
        Query::select()
            .expr(Expr::cust("COUNT(*) > 0"))
            .from("information_schema.tables")
            .cond_where(
                Condition::all()
                    .add(Expr::col("table_schema").eq(schema))
                    .add(Expr::col("table_name").eq(table)),
            ),
    )
    .await?
    .unwrap()
    .try_get_by_index(0)
    .map_err(DbErr::from)
}

#[cfg(feature = "sqlx-postgres")]
async fn pg_index_exists(db: &DatabaseConnection, table: &str, index: &str) -> Result<bool, DbErr> {
    db.query_one(
        Query::select()
            .expr(Expr::cust("COUNT(*) > 0"))
            .from("pg_indexes")
            .cond_where(
                Condition::all()
                    .add(Expr::cust("schemaname = CURRENT_SCHEMA()"))
                    .add(Expr::col("tablename").eq(table))
                    .add(Expr::col("indexname").eq(index)),
            ),
    )
    .await?
    .unwrap()
    .try_get_by_index(0)
    .map_err(DbErr::from)
}
