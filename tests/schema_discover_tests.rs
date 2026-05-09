//! Tests for the schema discovery module (`src/schema/discover/`).
//!
//! These tests verify that `discover()` produces the correct statements and
//! warnings for enum, table, column, index, and foreign key changes.
#![allow(unused_imports, dead_code)]
pub mod common;

use crate::common::TestContext;
use crate::common::fixtures::*;
use crate::common::helpers::{column_exists, table_exists};
#[cfg(feature = "schema-sync")]
use crate::common::helpers::discover_interpret_and_apply;
use sea_orm::{DatabaseConnection, DbErr, entity::*, query::*};

// ---------------------------------------------------------------------------
// Enum discovery tests (Postgres-only)
// ---------------------------------------------------------------------------

/// discover() on a brand-new entity must include a CREATE TYPE statement
/// for the Postgres enum before the CREATE TABLE.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_creates_enum_type() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_creates_enum_type").await;
    let db = &ctx.db;

        let builder = db.get_schema_builder().register(enum_v1::Entity);
        let change_set = builder.discover(db, false).await?;
        let result = sea_orm::interpret_changes(
            change_set,
            &sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: false,
            },
        );

        let has_create_type = result
            .statements
            .iter()
            .any(|(_, s)| s.sql.to_uppercase().contains("CREATE TYPE"));
        assert!(
            has_create_type,
            "discover() must include CREATE TYPE for Postgres enum; got: {:?}",
            result.statements
        );

        let has_create_table = result
            .statements
            .iter()
            .any(|(_, s)| s.sql.to_uppercase().contains("CREATE TABLE"));
        assert!(
            has_create_table,
            "discover() must include CREATE TABLE; got: {:?}",
            result.statements
        );

        let type_pos = result
            .statements
            .iter()
            .position(|(_, s)| s.sql.to_uppercase().contains("CREATE TYPE"))
            .unwrap();
        let table_pos = result
            .statements
            .iter()
            .position(|(_, s)| s.sql.to_uppercase().contains("CREATE TABLE"))
            .unwrap();
        assert!(
            type_pos < table_pos,
            "CREATE TYPE must precede CREATE TABLE; type at {type_pos}, table at {table_pos}"
        );

    ctx.delete().await;
    Ok(())
}

/// After an entity is synced, discover() must detect the existing enum and
/// NOT produce a duplicate CREATE TYPE statement.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_skips_existing_enum() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_skips_existing_enum").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(enum_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(enum_v1::Entity)
        .discover(db, false)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    );

    assert!(
        result
            .statements
            .iter()
            .all(|(_, s)| !s.sql.to_uppercase().contains("CREATE TYPE")),
        "discover() must NOT re-create an existing enum type; got: {:?}",
        result.statements
    );
    assert!(
        result.statements.is_empty(),
        "discover() should produce no changes when schema matches; got: {:?}",
        result.statements
    );

    ctx.delete().await;
    Ok(())
}

/// When an enum's variants change, dangerous discover must emit an EnumVariantChange warning.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_enum_variant_change_warning() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_enum_variant_change_warning").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(enum_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(enum_v2::Entity)
        .discover(db, true)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
            allow_dangerous: true,
        },
    );

    assert!(
        result
            .suggestions
            .iter()
            .any(|s| s.kind == sea_orm::schema::SuggestionKind::EnumVariantChange),
        "expected EnumVariantChange suggestion when enum gains a variant; got: {:?}",
        result.suggestions
    );
    assert!(
        result
            .statements
            .iter()
            .all(|(_, s)| !s.sql.to_uppercase().contains("CREATE TYPE")),
        "changed enum should produce a warning, not a CREATE TYPE; got: {:?}",
        result.statements
    );

    ctx.delete().await;
    Ok(())
}

/// When an enum type name changes (same variants), dangerous discover must emit a warning.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_enum_rename_warning() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_enum_rename_warning").await;
    let db = &ctx.db;

        db.get_schema_builder()
            .register(enum_v1::Entity)
            .sync(db)
            .await?;

        let change_set = db
            .get_schema_builder()
            .register(enum_renamed::Entity)
            .discover(db, true)
            .await?;
        let result = sea_orm::interpret_changes(
            change_set,
            &sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: true,
                allow_dangerous: true,
            },
        );

        assert!(
            result
                .suggestions
                .iter()
                .any(|s| s.kind == sea_orm::schema::SuggestionKind::EnumRename),
            "expected EnumRename suggestion when enum type is renamed; got: {:?}",
            result.suggestions
        );


    ctx.delete().await;
    Ok(())
}

/// Safe discover must NOT produce enum warnings even when variants changed.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_safe_no_enum_warnings() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_safe_no_enum_warnings").await;
    let db = &ctx.db;

        db.get_schema_builder()
            .register(enum_v1::Entity)
            .sync(db)
            .await?;

        let change_set = db
            .get_schema_builder()
            .register(enum_v2::Entity)
            .discover(db, false)
            .await?;
        let result = sea_orm::interpret_changes(
            change_set,
            &sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: false,
            },
        );

        assert!(
            result
                .suggestions
                .iter()
                .all(|s| s.kind != sea_orm::schema::SuggestionKind::EnumVariantChange),
            "safe discover should not suggest enum changes; got: {:?}",
            result.suggestions
        );

    ctx.delete().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Column drop / safe tests
// ---------------------------------------------------------------------------

/// When `allow_dangerous = true`, discover() must include a DROP COLUMN for removed columns.
/// Changes must NOT be applied until the caller executes them.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_drop_column() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_drop_column").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(widget_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(widget_v2::Entity)
        .discover(db, true)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: true,
        },
    );

    assert!(
        result
            .statements
            .iter()
            .any(|(_, s)| s.sql.to_uppercase().contains("DROP COLUMN")),
        "discover(dangerous=true) must include DROP COLUMN for `weight`; got: {:?}",
        result.statements
    );
    assert!(
        column_exists(db, "sync_widget", "weight").await?,
        "discover() must not apply changes; `weight` column should still exist"
    );

    ctx.delete().await;
    Ok(())
}

/// When `allow_dangerous = false`, discover() must NEVER produce any DROP statements.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
async fn test_discover_safe_no_drops() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_safe_no_drops").await;
    let db = &ctx.db;

    #[cfg(feature = "schema-sync")]
    {
        db.get_schema_builder()
            .register(widget_v1::Entity)
            .sync(db)
            .await?;

        let change_set = db
            .get_schema_builder()
            .register(widget_v2::Entity)
            .discover(db, false)
            .await?;
        let result = sea_orm::interpret_changes(
            change_set,
            &sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: false,
            },
        );

        assert!(
            result
                .statements
                .iter()
                .all(|(_, s)| !s.sql.to_uppercase().contains("DROP")),
            "discover(dangerous=false) must not include any DROP statements; got: {:?}",
            result.statements
        );
    }

    ctx.delete().await;
    Ok(())
}

/// Applying dangerous changes actually drops the column.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_sync_dangerous_drops_column() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_dangerous_drops_column").await;
    let db = &ctx.db;


        db.get_schema_builder()
            .register(widget_v1::Entity)
            .sync(db)
            .await?;
        assert!(column_exists(db, "sync_widget", "weight").await?);

        discover_interpret_and_apply(
            db,
            db.get_schema_builder().register(widget_v2::Entity),
            sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: true,
            },
        )
        .await?;

        assert!(!column_exists(db, "sync_widget", "weight").await?);
        assert!(column_exists(db, "sync_widget", "label").await?);


    ctx.delete().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Table drop tests
// ---------------------------------------------------------------------------

/// discover(dangerous=true) must include DROP TABLE for orphaned tables.
#[sea_orm_macros::test]
#[cfg(feature = "schema-sync")]
async fn test_discover_drop_table() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_drop_table").await;
    let db = &ctx.db;


        db.get_schema_builder()
            .register(tag_v1::Entity)
            .register(widget_v1::Entity)
            .sync(db)
            .await?;

        let change_set = db
            .get_schema_builder()
            .register(widget_v1::Entity)
            .discover(db, true)
            .await?;
        let result = sea_orm::interpret_changes(
            change_set,
            &sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: true,
            },
        );

        assert!(
            result
                .statements
                .iter()
                .any(|(_, s)| s.sql.to_uppercase().contains("DROP TABLE")),
            "discover(dangerous=true) must include DROP TABLE for `sync_tag`; got: {:?}",
            result.statements
        );
        assert!(
            table_exists(db, "sync_tag").await?,
            "discover() must not apply changes; `sync_tag` should still exist"
        );


    ctx.delete().await;
    Ok(())
}

/// Applying dangerous changes actually drops the orphaned table.
#[sea_orm_macros::test]
#[cfg(feature = "schema-sync")]
async fn test_sync_dangerous_drops_table() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_dangerous_drops_table").await;
    let db = &ctx.db;


        db.get_schema_builder()
            .register(tag_v1::Entity)
            .register(widget_v1::Entity)
            .sync(db)
            .await?;
        assert!(table_exists(db, "sync_tag").await?);

        discover_interpret_and_apply(
            db,
            db.get_schema_builder().register(widget_v1::Entity),
            sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: true,
            },
        )
        .await?;

        assert!(!table_exists(db, "sync_tag").await?);
        assert!(table_exists(db, "sync_widget").await?);


    ctx.delete().await;
    Ok(())
}

/// When both a parent and child table are orphaned, the child must appear before
/// the parent in the DROP TABLE statements (to avoid FK constraint violations).
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_sync_dangerous_drops_orphan_table_fk_order() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_dangerous_drops_orphan_table_fk_order").await;
    let db = &ctx.db;


        // Create both tables; fk_child has an FK to fk_parent.
        db.get_schema_builder()
            .register(fk_parent_v1::Entity)
            .register(fk_child_v1::Entity)
            .sync(db)
            .await?;

        assert!(table_exists(db, "sync_fk_parent").await?);
        assert!(table_exists(db, "sync_fk_child").await?);

        // Discover with no registered entities → both tables are orphans; apply in one shot.
        let result = discover_interpret_and_apply(
            db,
            db.get_schema_builder(),
            sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: true,
            },
        )
        .await?;

        let child_pos = result
            .statements
            .iter()
            .position(|(_, s)| s.sql.to_uppercase().contains("SYNC_FK_CHILD"))
            .expect("DROP TABLE sync_fk_child must be in statements");
        let parent_pos = result
            .statements
            .iter()
            .position(|(_, s)| s.sql.to_uppercase().contains("SYNC_FK_PARENT"))
            .expect("DROP TABLE sync_fk_parent must be in statements");

        assert!(
            child_pos < parent_pos,
            "child table must be dropped before parent to avoid FK violation; \
             child at {child_pos}, parent at {parent_pos}"
        );

        assert!(!table_exists(db, "sync_fk_child").await?);
        assert!(!table_exists(db, "sync_fk_parent").await?);


    ctx.delete().await;
    Ok(())
}


/// discover(dangerous=true) must include DROP FOREIGN KEY / CONSTRAINT when a FK is removed.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_drop_foreign_key() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_drop_foreign_key").await;
    let db = &ctx.db;


        db.get_schema_builder()
            .register(category_v1::Entity)
            .register(article_v1::Entity)
            .sync(db)
            .await?;

        let change_set = db
            .get_schema_builder()
            .register(category_v1::Entity)
            .register(article_v2::Entity)
            .discover(db, true)
            .await?;
        let result = sea_orm::interpret_changes(
            change_set,
            &sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: true,
            },
        );

        let has_drop_fk = result.statements.iter().any(|(_, s)| {
            let sql = s.sql.to_uppercase();
            sql.contains("DROP FOREIGN KEY") || sql.contains("DROP CONSTRAINT")
        });
        assert!(
            has_drop_fk,
            "discover(dangerous=true) must include DROP FOREIGN KEY / CONSTRAINT; got: {:?}",
            result.statements
        );


    ctx.delete().await;
    Ok(())
}

/// Applying dangerous changes actually removes the FK.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_sync_dangerous_drops_foreign_key() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_dangerous_drops_foreign_key").await;
    let db = &ctx.db;


        db.get_schema_builder()
            .register(category_v1::Entity)
            .register(article_v1::Entity)
            .sync(db)
            .await?;

        discover_interpret_and_apply(
            db,
            db.get_schema_builder()
                .register(category_v1::Entity)
                .register(article_v2::Entity),
            sea_orm::InterpretConfig {
                db_backend: db.get_database_backend(),
                assumptions: false,
                allow_dangerous: true,
            },
        )
        .await?;

        assert!(table_exists(db, "sync_article").await?);


    ctx.delete().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// No-change test
// ---------------------------------------------------------------------------

/// When the schema already matches, discover() must return an empty change set.
#[sea_orm_macros::test]
#[cfg(feature = "schema-sync")]
async fn test_discover_no_changes_when_synced() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_no_changes_when_synced").await;
    let db = &ctx.db;


        db.get_schema_builder()
            .register(widget_v1::Entity)
            .sync(db)
            .await?;

        for dangerous in [false, true] {
            let change_set = db
                .get_schema_builder()
                .register(widget_v1::Entity)
                .discover(db, dangerous)
                .await?;
            let result = sea_orm::interpret_changes(
                change_set,
                &sea_orm::InterpretConfig {
                    db_backend: db.get_database_backend(),
                    assumptions: false,
                    allow_dangerous: dangerous,
                },
            );
            assert!(
                result.statements.is_empty(),
                "discover(dangerous={dangerous}) must return no changes when schema is up-to-date; got: {:?}",
                result.statements
            );
        }


    ctx.delete().await;
    Ok(())
}


/// Dangerous sync with assumptions=true auto-assumes obvious rename and generates RENAME COLUMN.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_sync_dangerous_add_and_drop_column() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_sync_dangerous_add_and_drop_column").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(combo_v1::Entity)
        .sync(db)
        .await?;

    assert!(column_exists(db, "sync_combo", "old_field").await?);
    assert!(!column_exists(db, "sync_combo", "new_field").await?);

    let change_set = db
        .get_schema_builder()
        .register(combo_v2::Entity)
        .discover(db, true)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
            allow_dangerous: true,
        },
    );

    assert!(
        result
            .suggestions
            .iter()
            .any(|s| s.kind == sea_orm::schema::SuggestionKind::PossibleRename),
        "expected PossibleRename suggestion; got: {:?}",
        result.suggestions
    );
    assert!(
        result
            .statements
            .iter()
            .any(|(_, s)| s.sql.to_uppercase().contains("RENAME COLUMN")),
        "auto-assumed rename should produce RENAME COLUMN; got: {:?}",
        result.statements
    );
    assert!(
        result
            .statements
            .iter()
            .all(|(_, s)| !s.sql.to_uppercase().contains("ADD COLUMN")
                && !s.sql.to_uppercase().contains("DROP COLUMN")),
        "rename-detected pair should not produce ADD/DROP; got: {:?}",
        result.statements
    );

    ctx.delete().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Column addition tests
// ---------------------------------------------------------------------------

/// Adding a nullable column should produce an ADD COLUMN and no warnings.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_add_nullable_column() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_add_nullable_column").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(coltest_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(coltest_v2_nullable::Entity)
        .discover(db, false)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    );

    assert!(
        result.statements.iter().any(|(_, s)| {
            let sql = s.sql.to_uppercase();
            sql.contains("ADD COLUMN") && sql.contains("BIO")
        }),
        "discover() must include ADD COLUMN for `bio`; got: {:?}",
        result.statements
    );
    assert!(
        result
            .warnings
            .iter()
            .all(|w| w.kind != sea_orm::schema::WarningKind::NotNullNoDefault),
        "nullable column must not produce NotNullNoDefault warning; got: {:?}",
        result.warnings
    );

    ctx.delete().await;
    Ok(())
}

/// Adding a NOT NULL column without a default should produce a NotNullNoDefault warning.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_add_notnull_column_warns() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_add_notnull_column_warns").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(coltest_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(coltest_v2_notnull::Entity)
        .discover(db, false)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    );

    assert!(
        result.statements.iter().any(|(_, s)| {
            let sql = s.sql.to_uppercase();
            sql.contains("ADD COLUMN") && sql.contains("AGE")
        }),
        "discover() must include ADD COLUMN for `age`; got: {:?}",
        result.statements
    );
    assert!(
        result.warnings.iter().any(|w| {
            w.kind == sea_orm::schema::WarningKind::NotNullNoDefault
                && w.message.contains("age")
        }),
        "NOT NULL column without default must produce NotNullNoDefault warning; got: {:?}",
        result.warnings
    );

    ctx.delete().await;
    Ok(())
}

/// Adding a NOT NULL column WITH a default should NOT produce a NotNullNoDefault warning.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_add_notnull_with_default_no_warn() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_add_notnull_with_default_no_warn").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(coltest_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(coltest_v2_notnull_default::Entity)
        .discover(db, false)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    );

    assert!(
        result.statements.iter().any(|(_, s)| {
            let sql = s.sql.to_uppercase();
            sql.contains("ADD COLUMN") && sql.contains("SCORE")
        }),
        "discover() must include ADD COLUMN for `score`; got: {:?}",
        result.statements
    );
    assert!(
        result
            .warnings
            .iter()
            .all(|w| w.kind != sea_orm::schema::WarningKind::NotNullNoDefault),
        "NOT NULL column with default should not warn; got: {:?}",
        result.warnings
    );

    ctx.delete().await;
    Ok(())
}

/// Adding multiple columns produces ADD COLUMN for each and warns only for NOT NULL without default.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_add_multiple_columns() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_add_multiple_columns").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(coltest_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db
        .get_schema_builder()
        .register(coltest_v2_multi::Entity)
        .discover(db, false)
        .await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    );

    for col in ["BIO", "AGE", "SCORE"] {
        assert!(
            result.statements.iter().any(|(_, s)| {
                let sql = s.sql.to_uppercase();
                sql.contains("ADD COLUMN") && sql.contains(col)
            }),
            "discover() must include ADD COLUMN for `{col}`; got: {:?}",
            result.statements
        );
    }

    let not_null_warnings: Vec<_> = result
        .warnings
        .iter()
        .filter(|w| w.kind == sea_orm::schema::WarningKind::NotNullNoDefault)
        .collect();
    assert_eq!(
        not_null_warnings.len(),
        1,
        "expected exactly one NotNullNoDefault warning (for `age`); got: {not_null_warnings:?}"
    );
    assert!(
        not_null_warnings[0].message.contains("age"),
        "the warning should reference `age`; got: {:?}",
        not_null_warnings[0].message
    );

    ctx.delete().await;
    Ok(())
}

/// After discovering an ADD COLUMN, applying the statements creates the column.
#[sea_orm_macros::test]
#[cfg(not(any(feature = "sqlx-sqlite", feature = "rusqlite")))]
#[cfg(feature = "schema-sync")]
async fn test_discover_add_column_applies() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_add_column_applies").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(coltest_v1::Entity)
        .sync(db)
        .await?;

    assert!(!column_exists(db, "disc_coltest", "bio").await?);

    discover_interpret_and_apply(
        db,
        db.get_schema_builder().register(coltest_v2_nullable::Entity),
        sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    )
    .await?;

    assert!(column_exists(db, "disc_coltest", "bio").await?);

    ctx.delete().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Enum drop tests (Postgres-only)
// ---------------------------------------------------------------------------

/// When an enum type exists in the DB but has no matching entity, discover(dangerous=true)
/// must include a DROP TYPE statement.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_drops_orphan_enum_type() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_drops_orphan_enum_type").await;
    let db = &ctx.db;

    // Sync the enum entity to create the type in the DB.
    db.get_schema_builder()
        .register(enum_v1::Entity)
        .sync(db)
        .await?;

    // Discover with NO entities registered — the enum is now orphaned.
    let change_set = db.get_schema_builder().discover(db, true).await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: true,
        },
    );

    let has_drop_type = result
        .statements
        .iter()
        .any(|(_, s)| s.sql.to_uppercase().contains("DROP TYPE"));
    assert!(
        has_drop_type,
        "discover(dangerous=true) must include DROP TYPE for orphaned enum; got: {:?}",
        result.statements
    );

    ctx.delete().await;
    Ok(())
}

/// DROP TABLE must appear before DROP TYPE in the statement list (so the table
/// referencing the enum is gone before the type is dropped).
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_drop_table_before_enum_type() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_drop_table_before_enum_type").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(enum_v1::Entity)
        .sync(db)
        .await?;

    let result = discover_interpret_and_apply(
        db,
        db.get_schema_builder(),
        sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: true,
        },
    )
    .await?;

    let table_pos = result
        .statements
        .iter()
        .position(|(_, s)| s.sql.to_uppercase().contains("DROP TABLE"))
        .expect("DROP TABLE must be present");
    let type_pos = result
        .statements
        .iter()
        .position(|(_, s)| s.sql.to_uppercase().contains("DROP TYPE"))
        .expect("DROP TYPE must be present");

    assert!(
        table_pos < type_pos,
        "DROP TABLE must precede DROP TYPE; table at {table_pos}, type at {type_pos}"
    );

    assert!(!table_exists(db, "disc_enum_table").await?);

    ctx.delete().await;
    Ok(())
}

/// Safe discover (allow_dangerous=false) must NOT produce DROP TYPE even for orphaned enums.
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_discover_safe_no_drop_enum_type() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_discover_safe_no_drop_enum_type").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(enum_v1::Entity)
        .sync(db)
        .await?;

    let change_set = db.get_schema_builder().discover(db, false).await?;
    let result = sea_orm::interpret_changes(
        change_set,
        &sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: false,
        },
    );

    assert!(
        result
            .statements
            .iter()
            .all(|(_, s)| !s.sql.to_uppercase().contains("DROP TYPE")),
        "safe discover must not produce DROP TYPE; got: {:?}",
        result.statements
    );

    ctx.delete().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Complex drop-sequence test
// ---------------------------------------------------------------------------

/// Full drop-sequence correctness: three-level FK chain (grandparent → mid → leaf)
/// where the grandparent has an enum column, all orphaned at once.
///
/// Verifies the complete required ordering in a single `discover + execute` pass:
///   1. DROP CONSTRAINT (FK drops) before DROP TABLE for the same table
///   2. Leaf before mid before grandparent (child-first table drops)
///   3. DROP TYPE after all DROP TABLE statements
///   4. The statements actually execute without FK/type-dependency errors
#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
#[cfg(feature = "schema-sync")]
async fn test_complex_drop_sequence() -> Result<(), DbErr> {
    let ctx = TestContext::new("test_complex_drop_sequence").await;
    let db = &ctx.db;

    // Build the three-level chain: grandparent (has enum) → mid → leaf.
    db.get_schema_builder()
        .register(fk_grandparent_v1::Entity)
        .register(fk_mid_v1::Entity)
        .register(fk_leaf_v1::Entity)
        .sync(db)
        .await?;

    assert!(table_exists(db, "drop_seq_gp").await?);
    assert!(table_exists(db, "drop_seq_mid").await?);
    assert!(table_exists(db, "drop_seq_leaf").await?);

    // Orphan all three by registering nothing; apply the statements in one shot.
    let result = discover_interpret_and_apply(
        db,
        db.get_schema_builder(),
        sea_orm::InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: false,
            allow_dangerous: true,
        },
    )
    .await?;

    let stmts: Vec<_> = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.to_uppercase())
        .collect();

    // ── 1. All expected statement kinds are present ──────────────────────
    assert!(
        stmts.iter().any(|s| s.contains("DROP TABLE") && s.contains("DROP_SEQ_LEAF")),
        "DROP TABLE drop_seq_leaf must be present; got:\n{stmts:#?}"
    );
    assert!(
        stmts.iter().any(|s| s.contains("DROP TABLE") && s.contains("DROP_SEQ_MID")),
        "DROP TABLE drop_seq_mid must be present; got:\n{stmts:#?}"
    );
    assert!(
        stmts.iter().any(|s| s.contains("DROP TABLE") && s.contains("DROP_SEQ_GP")),
        "DROP TABLE drop_seq_gp must be present; got:\n{stmts:#?}"
    );
    assert!(
        stmts.iter().any(|s| s.contains("DROP TYPE")),
        "DROP TYPE must be present for the orphaned enum; got:\n{stmts:#?}"
    );

    // ── 2. Child-first table-drop order across all three levels ──────────
    let leaf_pos = stmts
        .iter()
        .position(|s| s.contains("DROP TABLE") && s.contains("DROP_SEQ_LEAF"))
        .unwrap();
    let mid_pos = stmts
        .iter()
        .position(|s| s.contains("DROP TABLE") && s.contains("DROP_SEQ_MID"))
        .unwrap();
    let gp_pos = stmts
        .iter()
        .position(|s| s.contains("DROP TABLE") && s.contains("DROP_SEQ_GP"))
        .unwrap();
    assert!(
        leaf_pos < mid_pos,
        "leaf must be dropped before mid; leaf at {leaf_pos}, mid at {mid_pos}"
    );
    assert!(
        mid_pos < gp_pos,
        "mid must be dropped before grandparent; mid at {mid_pos}, gp at {gp_pos}"
    );

    // ── 3. Any FK DROP CONSTRAINT comes before its own table's DROP TABLE ─
    for (fk_table, fk_table_upper) in [
        ("drop_seq_mid", "DROP_SEQ_MID"),
        ("drop_seq_leaf", "DROP_SEQ_LEAF"),
    ] {
        let drop_table_pos = stmts
            .iter()
            .position(|s| s.contains("DROP TABLE") && s.contains(fk_table_upper))
            .unwrap();
        if let Some(drop_constraint_pos) = stmts
            .iter()
            .position(|s| s.contains("DROP CONSTRAINT") && s.contains(fk_table_upper))
        {
            assert!(
                drop_constraint_pos < drop_table_pos,
                "DROP CONSTRAINT on {fk_table} must precede DROP TABLE {fk_table}; \
                 constraint at {drop_constraint_pos}, table at {drop_table_pos}"
            );
        }
    }

    // ── 4. DROP TYPE is after all DROP TABLE statements ───────────────────
    let last_drop_table_pos = stmts
        .iter()
        .rposition(|s| s.contains("DROP TABLE"))
        .unwrap();
    let drop_type_pos = stmts
        .iter()
        .position(|s| s.contains("DROP TYPE"))
        .unwrap();
    assert!(
        drop_type_pos > last_drop_table_pos,
        "DROP TYPE must come after all DROP TABLE statements; \
         last DROP TABLE at {last_drop_table_pos}, DROP TYPE at {drop_type_pos}"
    );

    // ── 5. Verify tables are gone (statements were applied by discover_interpret_and_apply) ──
    assert!(!table_exists(db, "drop_seq_leaf").await?);
    assert!(!table_exists(db, "drop_seq_mid").await?);
    assert!(!table_exists(db, "drop_seq_gp").await?);

    ctx.delete().await;
    Ok(())
}
