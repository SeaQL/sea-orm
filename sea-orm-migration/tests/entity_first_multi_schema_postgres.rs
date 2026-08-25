//! Regression tests for entity-first `discover`/`sync` across multiple
//! non-default PostgreSQL schemas. `SchemaBuilder::discover` only ever
//! queried the connection's current schema, so every entity pinned to a
//! `schema_name` other than `public` was reported as missing on every run —
//! `sync` would try (and fail) to re-create it, and `discover` could never
//! see real diffs (added columns, etc.) for tables living outside `public`.
//!
//! Requires a real PostgreSQL connection via `DATABASE_URL`, since SQLite has
//! no concept of multiple schemas.

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};
use sea_orm_migration::{EntitySet, SchemaBuilder};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// `widget` v1, pinned to schema `entity_first_schema_a`.
mod widget_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_schema_a", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `widget` v2 — adds a `description` column, same schema + table as v1.
mod widget_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_schema_a", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub description: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `gadget`, pinned to a different non-default schema, `entity_first_schema_b`.
mod gadget {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_schema_b", table_name = "gadget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub label: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `thing` — no `schema_name`, lives in the connection's default (`public`) schema.
mod thing {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "thing")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub value: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Also named `widget`, but pinned to schema `entity_first_schema_b` with a
/// different column set. Used to prove discovery doesn't conflate same-named
/// tables living in different schemas.
mod widget_in_schema_b {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_schema_b", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub tag: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Same table name (`widget`) and same columns as `widget_v1`, but pinned to
/// `entity_first_schema_b` instead of `entity_first_schema_a` — a pure
/// schema move (no rename), with an identical column signature so table-move
/// detection should actually match it (unlike `widget_in_schema_b` above).
mod widget_moved {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_schema_b", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Same columns as `widget_v1`, but both a different table name (`sprocket`)
/// *and* a different schema (`entity_first_schema_b`) — a combined
/// rename + schema move.
mod sprocket_renamed_and_moved {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_schema_b", table_name = "sprocket")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

struct FullSet;
impl EntitySet for FullSet {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder
            .register(widget_v1::Entity)
            .register(gadget::Entity)
            .register(thing::Entity)
    }
}

struct FullSetV2;
impl EntitySet for FullSetV2 {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder
            .register(widget_v2::Entity)
            .register(gadget::Entity)
            .register(thing::Entity)
    }
}

struct CollisionSet;
impl EntitySet for CollisionSet {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder {
        builder
            .register(widget_v1::Entity)
            .register(widget_in_schema_b::Entity)
    }
}

// ---------------------------------------------------------------------------
// DB setup helpers
// ---------------------------------------------------------------------------

/// Connects to a fresh, empty database (dropped + recreated) with the two
/// non-default schemas used by these tests already in place.
async fn fresh_multi_schema_db(db_name: &str) -> Result<DatabaseConnection, DbErr> {
    let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
    // Strip the existing `/<database>` path segment (if any) so we can swap in `db_name`.
    let base_url = match url.rfind('/') {
        Some(pos) if pos > "postgres://".len() => url[..pos].to_owned(),
        _ => url.clone(),
    };

    let root = Database::connect(ConnectOptions::new(url.clone())).await?;
    root.execute_unprepared(&format!(r#"DROP DATABASE IF EXISTS "{db_name}""#))
        .await?;
    root.execute_unprepared(&format!(r#"CREATE DATABASE "{db_name}""#))
        .await?;

    let db = Database::connect(ConnectOptions::new(format!("{base_url}/{db_name}"))).await?;
    db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS entity_first_schema_a")
        .await?;
    db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS entity_first_schema_b")
        .await?;
    Ok(db)
}

/// Connects to a fresh, empty database (dropped + recreated) with *no*
/// schemas pre-created — used to verify `discover`/`sync` provision the
/// namespaces themselves rather than assuming they already exist.
async fn fresh_db_no_schemas(db_name: &str) -> Result<DatabaseConnection, DbErr> {
    let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
    let base_url = match url.rfind('/') {
        Some(pos) if pos > "postgres://".len() => url[..pos].to_owned(),
        _ => url.clone(),
    };

    let root = Database::connect(ConnectOptions::new(url.clone())).await?;
    root.execute_unprepared(&format!(r#"DROP DATABASE IF EXISTS "{db_name}""#))
        .await?;
    root.execute_unprepared(&format!(r#"CREATE DATABASE "{db_name}""#))
        .await?;

    Database::connect(ConnectOptions::new(format!("{base_url}/{db_name}"))).await
}

async fn schema_exists(db: &DatabaseConnection, schema: &str) -> Result<bool, DbErr> {
    use sea_orm::ExprTrait;
    use sea_orm::sea_query::{Expr, Query};

    let row = db
        .query_one(
            &Query::select()
                .expr(Expr::cust("COUNT(*) > 0"))
                .from(("information_schema", "schemata"))
                .and_where(Expr::col("schema_name").eq(schema))
                .to_owned(),
        )
        .await?
        .unwrap();
    row.try_get_by_index(0).map_err(DbErr::from)
}

async fn table_exists_in_schema(
    db: &DatabaseConnection,
    schema: &str,
    table: &str,
) -> Result<bool, DbErr> {
    use sea_orm::ExprTrait;
    use sea_orm::sea_query::{Condition, Expr, Query};

    let row = db
        .query_one(
            &Query::select()
                .expr(Expr::cust("COUNT(*) > 0"))
                .from(("information_schema", "tables"))
                .cond_where(
                    Condition::all()
                        .add(Expr::col("table_schema").eq(schema))
                        .add(Expr::col("table_name").eq(table)),
                )
                .to_owned(),
        )
        .await?
        .unwrap();
    row.try_get_by_index(0).map_err(DbErr::from)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// `discover` on an empty (but schema-created) database must propose creating
/// every table, each qualified with its entity's own `schema_name`.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_discover_multi_schema_creates_tables_in_correct_schemas() -> Result<(), DbErr> {
    let db = fresh_multi_schema_db("entity_first_multi_schema_discover").await?;

    let builder = FullSet.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db).await?;
    let stmts = change_set.statements();

    let creates: Vec<_> = stmts
        .iter()
        .filter(|s| s.sql.to_uppercase().contains("CREATE TABLE"))
        .collect();
    assert_eq!(
        creates.len(),
        3,
        "should create widget + gadget + thing, got: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );

    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains(r#""entity_first_schema_a"."widget""#),
        "widget should be qualified with schema_a: {sql_all}"
    );
    assert!(
        sql_all.contains(r#""entity_first_schema_b"."gadget""#),
        "gadget should be qualified with schema_b: {sql_all}"
    );
    assert!(
        sql_all.contains(r#""thing""#) && !sql_all.contains(r#"."thing""#),
        "thing should be unqualified (default schema): {sql_all}"
    );

    Ok(())
}

/// Regression test: syncing an entity set spanning multiple non-default
/// schemas twice must not fail with "relation already exists" — the second
/// sync must correctly see the first sync's tables regardless of schema.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_sync_multi_schema_twice_is_idempotent() -> Result<(), DbErr> {
    let db = fresh_multi_schema_db("entity_first_multi_schema_sync").await?;

    FullSet
        .register(Schema::new(db.get_database_backend()).builder())
        .sync(&db)
        .await?;

    // Must not error with "relation already exists" for the schema_a/schema_b tables.
    FullSet
        .register(Schema::new(db.get_database_backend()).builder())
        .sync(&db)
        .await?;

    assert!(table_exists_in_schema(&db, "entity_first_schema_a", "widget").await?);
    assert!(table_exists_in_schema(&db, "entity_first_schema_b", "gadget").await?);
    assert!(table_exists_in_schema(&db, "public", "thing").await?);

    // A subsequent discover should now see no diff at all.
    let builder = FullSet.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db).await?;
    let stmts = change_set.statements();
    assert!(
        stmts.is_empty(),
        "no changes expected after two syncs, got: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );

    Ok(())
}

/// A column added to an entity pinned to a non-default schema must be
/// detected as an `ADD COLUMN` against the existing table, not as a brand
/// new `CREATE TABLE` (which is what happens if discovery can't see the
/// table because it only looked in the default schema).
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_discover_detects_added_column_in_non_default_schema() -> Result<(), DbErr> {
    let db = fresh_multi_schema_db("entity_first_multi_schema_column").await?;

    Schema::new(db.get_database_backend())
        .builder()
        .register(widget_v1::Entity)
        .sync(&db)
        .await?;

    let builder = FullSetV2.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db).await?;
    let stmts = change_set.statements();

    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.to_uppercase().contains("ADD COLUMN") && sql_all.contains("description"),
        "should ADD COLUMN description on the existing schema_a.widget, got: {sql_all}"
    );
    assert!(
        !stmts
            .iter()
            .any(|s| s.sql.to_uppercase().contains("CREATE TABLE") && s.sql.contains("widget")),
        "widget must not be re-created, only altered: {sql_all}"
    );

    Ok(())
}

/// Two entities that share a table name but live in different schemas must
/// not be conflated by discovery — each is diffed against its own schema's
/// table, so a synced pair produces no spurious diff.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_discover_no_cross_schema_name_collision() -> Result<(), DbErr> {
    let db = fresh_multi_schema_db("entity_first_multi_schema_collision").await?;

    CollisionSet
        .register(Schema::new(db.get_database_backend()).builder())
        .sync(&db)
        .await?;

    assert!(table_exists_in_schema(&db, "entity_first_schema_a", "widget").await?);
    assert!(table_exists_in_schema(&db, "entity_first_schema_b", "widget").await?);

    let builder = CollisionSet.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db).await?;
    let stmts = change_set.statements();
    assert!(
        stmts.is_empty(),
        "same-named tables in different schemas must not produce a spurious diff, got: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );

    Ok(())
}

/// Regression test: `discover`/`sync` must provision a missing non-default
/// PostgreSQL schema themselves — `entity schema` / `discover` previously
/// never emitted `CREATE SCHEMA`, so `sync` on a brand new database (where
/// the target schema doesn't exist yet) failed with
/// `ERROR: schema "entity_first_schema_a" does not exist`.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_discover_and_sync_create_missing_schema_from_scratch() -> Result<(), DbErr> {
    let db = fresh_db_no_schemas("entity_first_multi_schema_missing_schema").await?;

    assert!(!schema_exists(&db, "entity_first_schema_a").await?);
    assert!(!schema_exists(&db, "entity_first_schema_b").await?);

    let builder = FullSet.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db).await?;
    let stmts = change_set.statements();
    assert!(
        stmts
            .first()
            .is_some_and(|s| s.sql.to_uppercase().contains("CREATE SCHEMA")),
        "discover should lead with CREATE SCHEMA statements, got: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );
    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains(r#"CREATE SCHEMA IF NOT EXISTS "entity_first_schema_a""#),
        "should create schema_a: {sql_all}"
    );
    assert!(
        sql_all.contains(r#"CREATE SCHEMA IF NOT EXISTS "entity_first_schema_b""#),
        "should create schema_b: {sql_all}"
    );

    // Must not error with "schema does not exist".
    FullSet
        .register(Schema::new(db.get_database_backend()).builder())
        .sync(&db)
        .await?;

    assert!(schema_exists(&db, "entity_first_schema_a").await?);
    assert!(schema_exists(&db, "entity_first_schema_b").await?);
    assert!(table_exists_in_schema(&db, "entity_first_schema_a", "widget").await?);
    assert!(table_exists_in_schema(&db, "entity_first_schema_b", "gadget").await?);

    Ok(())
}

/// Regression test: `entity diff` / `entity generate` (the CLI commands a
/// user actually runs) go through `interpret_changes`, a completely separate
/// code path from `ChangeSet::statements()` used by `sync()`. That path
/// didn't know about the new schema-creation changes at all, so `entity
/// diff` never listed "Created schema: ..." and the migration file
/// `entity generate` writes never contained the `CREATE SCHEMA` statement —
/// even though `sync()` itself worked. This exercises that exact path.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_interpret_changes_includes_missing_schema() -> Result<(), DbErr> {
    use sea_orm::{InterpretConfig, interpret_changes};
    use sea_orm_migration::summary::summarize;

    let db = fresh_db_no_schemas("entity_first_multi_schema_interpret").await?;

    let builder = FullSet.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db).await?;
    let result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
            allow_dangerous: false,
        },
    );

    let stmts: Vec<sea_orm::Statement> = result.statements.iter().map(|(_, s)| s.clone()).collect();
    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains(r#"CREATE SCHEMA IF NOT EXISTS "entity_first_schema_a""#),
        "interpret_changes() must include CREATE SCHEMA for schema_a: {sql_all}"
    );
    assert!(
        sql_all.contains(r#"CREATE SCHEMA IF NOT EXISTS "entity_first_schema_b""#),
        "interpret_changes() must include CREATE SCHEMA for schema_b: {sql_all}"
    );
    assert!(
        stmts
            .first()
            .is_some_and(|s| s.sql.to_uppercase().contains("CREATE SCHEMA")),
        "CREATE SCHEMA must be the first statement, got: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );

    let changes = summarize(&stmts);
    assert!(
        changes.contains(&"Created schema: entity_first_schema_a".to_string()),
        "the 'Changes' summary shown by `entity diff` must list the schema creation, got: {changes:?}"
    );
    assert!(
        changes.contains(&"Created schema: entity_first_schema_b".to_string()),
        "got: {changes:?}"
    );

    Ok(())
}

/// Regression test: renaming an entity's `schema_name` (schema_a -> schema_b)
/// leaves a table behind that no registered entity references anymore.
/// Dangerous discovery must scan every schema in the database — not just
/// ones a *current* entity references — so it can still find and flag that
/// orphan for a DROP TABLE.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_discover_dangerous_detects_orphan_after_schema_rename() -> Result<(), DbErr> {
    use sea_orm::{InterpretConfig, interpret_changes};

    let db = fresh_multi_schema_db("entity_first_multi_schema_rename").await?;

    // widget starts out in schema_a.
    Schema::new(db.get_database_backend())
        .builder()
        .register(widget_v1::Entity)
        .sync(&db)
        .await?;
    assert!(table_exists_in_schema(&db, "entity_first_schema_a", "widget").await?);

    // The entity now points at schema_b instead (same table name, simulating
    // a `schema_name` rename) — schema_a's copy is no longer referenced by
    // any entity.
    let builder = Schema::new(db.get_database_backend())
        .builder()
        .register(widget_in_schema_b::Entity);
    let change_set = builder.discover(&db).await?;
    let result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
            allow_dangerous: true,
        },
    );
    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        sql_all.to_uppercase().contains("DROP TABLE")
            && sql_all.contains("entity_first_schema_a")
            && sql_all.contains("widget"),
        "should detect the orphaned schema_a.widget and propose dropping it, got: {sql_all}"
    );

    Ok(())
}

/// A table moved to a different schema with the same name and an identical
/// column signature must be detected as a pure schema move
/// (`ALTER TABLE ... SET SCHEMA`), not a DROP + CREATE pair.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_table_schema_move_detected() -> Result<(), DbErr> {
    use sea_orm::{InterpretConfig, interpret_changes};

    let db = fresh_multi_schema_db("entity_first_multi_schema_move").await?;

    Schema::new(db.get_database_backend())
        .builder()
        .register(widget_v1::Entity)
        .sync(&db)
        .await?;
    assert!(table_exists_in_schema(&db, "entity_first_schema_a", "widget").await?);

    let builder = Schema::new(db.get_database_backend())
        .builder()
        .register(widget_moved::Entity);
    let change_set = builder.discover(&db).await?;
    let result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
            allow_dangerous: true,
        },
    );

    assert_eq!(
        result.table_moves.len(),
        1,
        "should detect exactly one schema move, got: {:?}",
        result.table_moves
    );
    assert_eq!(
        result.table_moves[0].from_name(),
        "entity_first_schema_a.widget"
    );
    assert_eq!(
        result.table_moves[0].to_name(),
        "entity_first_schema_b.widget"
    );

    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.to_uppercase().contains("SET SCHEMA")
            && sql_all.contains("entity_first_schema_a")
            && sql_all.contains("entity_first_schema_b")
            && sql_all.contains("widget"),
        "should produce ALTER TABLE ... SET SCHEMA, got: {sql_all}"
    );
    assert!(
        !sql_all.to_uppercase().contains("DROP TABLE")
            && !sql_all.to_uppercase().contains("CREATE TABLE"),
        "should not fall back to CREATE+DROP, got: {sql_all}"
    );

    Ok(())
}

/// A table renamed *and* moved to a different schema at once, with an
/// identical column signature, must be detected as a single combined move
/// (RENAME TO, then SET SCHEMA), not a DROP + CREATE pair.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_table_rename_and_schema_move_detected() -> Result<(), DbErr> {
    use sea_orm::{InterpretConfig, interpret_changes};

    let db = fresh_multi_schema_db("entity_first_multi_schema_rename_move").await?;

    Schema::new(db.get_database_backend())
        .builder()
        .register(widget_v1::Entity)
        .sync(&db)
        .await?;
    assert!(table_exists_in_schema(&db, "entity_first_schema_a", "widget").await?);

    let builder = Schema::new(db.get_database_backend())
        .builder()
        .register(sprocket_renamed_and_moved::Entity);
    let change_set = builder.discover(&db).await?;
    let result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
            allow_dangerous: true,
        },
    );

    assert_eq!(
        result.table_moves.len(),
        1,
        "should detect exactly one combined move, got: {:?}",
        result.table_moves
    );
    assert_eq!(
        result.table_moves[0].from_name(),
        "entity_first_schema_a.widget"
    );
    assert_eq!(
        result.table_moves[0].to_name(),
        "entity_first_schema_b.sprocket"
    );

    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.to_uppercase().contains("RENAME") && sql_all.contains("sprocket"),
        "should produce a RENAME statement, got: {sql_all}"
    );
    assert!(
        sql_all.to_uppercase().contains("SET SCHEMA") && sql_all.contains("entity_first_schema_b"),
        "should produce a SET SCHEMA statement, got: {sql_all}"
    );
    assert!(
        !sql_all.to_uppercase().contains("DROP TABLE")
            && !sql_all.to_uppercase().contains("CREATE TABLE"),
        "should not fall back to CREATE+DROP, got: {sql_all}"
    );

    Ok(())
}

/// `.exclude_schema(...)` must keep dangerous discovery from ever scanning
/// (and therefore proposing drops for) a schema that belongs to another
/// application/tenant sharing the same database.
#[tokio::test]
#[cfg(feature = "sqlx-postgres")]
async fn test_exclude_schema_protects_foreign_schema_from_orphan_scan() -> Result<(), DbErr> {
    let db = fresh_multi_schema_db("entity_first_multi_schema_exclude").await?;

    db.execute_unprepared("CREATE SCHEMA IF NOT EXISTS entity_first_foreign_schema")
        .await?;
    db.execute_unprepared(
        r#"CREATE TABLE entity_first_foreign_schema.foreign_table ("id" integer NOT NULL)"#,
    )
    .await?;

    // Without exclude_schema: the foreign app's table is scanned and shows up as an orphan.
    let change_set = Schema::new(db.get_database_backend())
        .builder()
        .register(thing::Entity)
        .discover(&db)
        .await?;
    let sql_all: String = change_set
        .tables
        .iter()
        .map(|tc| format!("{:?}", tc.kind))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains("entity_first_foreign_schema") && sql_all.contains("foreign_table"),
        "sanity check: without exclude_schema, the foreign table should be seen at all, got: {sql_all}"
    );

    // With exclude_schema: the foreign schema must never be scanned at all.
    let change_set = Schema::new(db.get_database_backend())
        .builder()
        .register(thing::Entity)
        .exclude_schema("entity_first_foreign_schema")
        .discover(&db)
        .await?;
    let sql_all: String = change_set
        .tables
        .iter()
        .map(|tc| format!("{:?}", tc.kind))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        !sql_all.contains("entity_first_foreign_schema") && !sql_all.contains("foreign_table"),
        "excluded schema must never be scanned or proposed for drops, got: {sql_all}"
    );

    Ok(())
}
