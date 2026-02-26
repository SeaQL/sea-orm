#![allow(unused_imports, dead_code)]

pub mod common;

use crate::common::TestContext;
use sea_orm::{
    DatabaseConnection, DbErr,
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
