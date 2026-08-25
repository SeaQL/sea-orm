//! MySQL analogue of `entity_first_multi_schema.rs`. On MySQL, `schema_name`
//! addresses a separate *database* on the same server rather than a
//! namespace inside one database, so these tests exercise the
//! cross-database discovery path added alongside the PostgreSQL support,
//! live against a real MySQL server.
//!
//! Unlike PostgreSQL, `discover`/`sync` do not auto-create a missing
//! `schema_name` database on MySQL (creating a database is out of scope for
//! a table-sync tool) — so unlike the Postgres suite, there is no
//! "create missing schema from scratch" test here; every database an entity
//! references must already exist.
//!
//! Also unlike PostgreSQL, dangerous discovery never scans every database on
//! the server: a MySQL server commonly hosts other applications'/tests'
//! databases, and there is no safe way to tell those apart from ones this
//! app used to own. So MySQL orphan detection is limited to databases a
//! *currently* registered entity's `schema_name` points at — a `schema_name`
//! rename leaves the old database untouched rather than risking a scan (and
//! potential drops) across the whole server.
//!
//! Requires a real MySQL connection via `DATABASE_URL`.

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};
use sea_orm_migration::{EntitySet, SchemaBuilder};

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

/// `widget` v1, pinned to database `entity_first_mysql_schema_a`.
mod widget_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_mysql_schema_a", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `widget` v2 — adds a `description` column, same database + table as v1.
mod widget_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_mysql_schema_a", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub description: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `gadget`, pinned to a different non-default database, `entity_first_mysql_schema_b`.
mod gadget {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_mysql_schema_b", table_name = "gadget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub label: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `thing` — no `schema_name`, lives in the connection's own (default) database.
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

/// Also named `widget`, but pinned to `entity_first_mysql_schema_b`, used to
/// simulate a `schema_name` rename away from `entity_first_mysql_schema_a`.
mod widget_in_schema_b {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(schema_name = "entity_first_mysql_schema_b", table_name = "widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub tag: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// `origin` — no `schema_name`, used as the "before" side of a same-database
/// table rename test (MySQL builds renames via `RENAME TABLE`, unlike
/// Postgres/SQLite's `ALTER TABLE ... RENAME TO`).
mod origin_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "origin")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Same columns as `origin_v1`, different table name, same (default)
/// database — the "after" side of the rename.
mod destination {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "destination")]
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

// ---------------------------------------------------------------------------
// DB setup helpers
// ---------------------------------------------------------------------------

const SCHEMA_A: &str = "entity_first_mysql_schema_a";
const SCHEMA_B: &str = "entity_first_mysql_schema_b";

/// `widget_v1`/`widget_v2`/`gadget`/`widget_in_schema_b` all pin their
/// `schema_name` to the two literal databases above at compile time, so
/// unlike `db_name` they can't be made unique per test. Every test that goes
/// through `fresh_multi_schema_db` must hold this lock for its entire body —
/// otherwise two tests running in parallel (the default) race to drop/recreate
/// the same shared `SCHEMA_A`/`SCHEMA_B` databases out from under each other.
static SCHEMA_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Connects to a fresh, empty database (dropped + recreated) with the two
/// non-default "schema" databases used by these tests also dropped + recreated.
async fn fresh_multi_schema_db(db_name: &str) -> Result<DatabaseConnection, DbErr> {
    let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
    let base_url = match url.rfind('/') {
        Some(pos) if pos > "mysql://".len() => url[..pos].to_owned(),
        _ => url.clone(),
    };

    let root = Database::connect(ConnectOptions::new(url.clone())).await?;
    for name in [db_name, SCHEMA_A, SCHEMA_B] {
        root.execute_unprepared(&format!("DROP DATABASE IF EXISTS `{name}`"))
            .await?;
        root.execute_unprepared(&format!("CREATE DATABASE `{name}`"))
            .await?;
    }

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

/// `discover` on empty (but pre-created) databases must propose creating
/// every table, each qualified against its entity's own `schema_name` database.
#[tokio::test]
#[cfg(feature = "sqlx-mysql")]
async fn test_discover_multi_schema_creates_tables_in_correct_databases() -> Result<(), DbErr> {
    let _guard = SCHEMA_LOCK.lock().await;
    let db = fresh_multi_schema_db("entity_first_mysql_multi_schema_discover").await?;

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
        sql_all.contains(&format!("`{SCHEMA_A}`.`widget`")),
        "widget should be qualified with schema_a: {sql_all}"
    );
    assert!(
        sql_all.contains(&format!("`{SCHEMA_B}`.`gadget`")),
        "gadget should be qualified with schema_b: {sql_all}"
    );

    Ok(())
}

/// Regression test: syncing an entity set spanning multiple non-default
/// "schema" databases twice must not fail with "table already exists".
#[tokio::test]
#[cfg(feature = "sqlx-mysql")]
async fn test_sync_multi_schema_twice_is_idempotent() -> Result<(), DbErr> {
    let _guard = SCHEMA_LOCK.lock().await;
    let db = fresh_multi_schema_db("entity_first_mysql_multi_schema_sync").await?;

    FullSet
        .register(Schema::new(db.get_database_backend()).builder())
        .sync(&db)
        .await?;

    // Must not error with "table already exists" for the schema_a/schema_b tables.
    FullSet
        .register(Schema::new(db.get_database_backend()).builder())
        .sync(&db)
        .await?;

    assert!(table_exists_in_schema(&db, SCHEMA_A, "widget").await?);
    assert!(table_exists_in_schema(&db, SCHEMA_B, "gadget").await?);
    assert!(table_exists_in_schema(&db, "entity_first_mysql_multi_schema_sync", "thing").await?);

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

/// A column added to an entity pinned to a non-default "schema" database
/// must be detected as an `ADD COLUMN` against the existing table, not as a
/// brand new `CREATE TABLE`.
#[tokio::test]
#[cfg(feature = "sqlx-mysql")]
async fn test_discover_detects_added_column_in_non_default_schema() -> Result<(), DbErr> {
    let _guard = SCHEMA_LOCK.lock().await;
    let db = fresh_multi_schema_db("entity_first_mysql_multi_schema_column").await?;

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

/// Regression test: renaming an entity's `schema_name` (schema_a -> schema_b)
/// leaves a table behind that no registered entity references anymore.
/// Unlike PostgreSQL, dangerous discovery must NOT scan every database on
/// the server looking for it — schema_a is a database that could belong to
/// another application entirely, so it must be left untouched rather than
/// scanned (and potentially proposed for a drop).
#[tokio::test]
#[cfg(feature = "sqlx-mysql")]
async fn test_discover_dangerous_ignores_orphan_after_schema_rename() -> Result<(), DbErr> {
    use sea_orm::{InterpretConfig, interpret_changes};

    let _guard = SCHEMA_LOCK.lock().await;
    let db = fresh_multi_schema_db("entity_first_mysql_multi_schema_rename").await?;

    // widget starts out in schema_a.
    Schema::new(db.get_database_backend())
        .builder()
        .register(widget_v1::Entity)
        .sync(&db)
        .await?;
    assert!(table_exists_in_schema(&db, SCHEMA_A, "widget").await?);

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
        },
    );
    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        !sql_all.to_uppercase().contains("DROP TABLE") || !sql_all.contains(SCHEMA_A),
        "schema_a is no longer referenced by any entity and must not be scanned or dropped, got: {sql_all}"
    );
    // The old table is still there, untouched.
    assert!(table_exists_in_schema(&db, SCHEMA_A, "widget").await?);

    Ok(())
}

/// A table renamed within the same (default) database, with an identical
/// column signature, must be detected via MySQL's `RENAME TABLE` syntax
/// rather than falling back to DROP + CREATE.
#[tokio::test]
#[cfg(feature = "sqlx-mysql")]
async fn test_table_rename_detected_same_database() -> Result<(), DbErr> {
    use sea_orm::{InterpretConfig, interpret_changes};

    let _guard = SCHEMA_LOCK.lock().await;
    let db = fresh_multi_schema_db("entity_first_mysql_rename").await?;

    Schema::new(db.get_database_backend())
        .builder()
        .register(origin_v1::Entity)
        .sync(&db)
        .await?;

    let builder = Schema::new(db.get_database_backend())
        .builder()
        .register(destination::Entity);
    let change_set = builder.discover(&db).await?;
    let result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: db.get_database_backend(),
            assumptions: true,
        },
    );

    assert_eq!(
        result.table_moves.len(),
        1,
        "should detect exactly one table rename, got: {:?}",
        result.table_moves
    );
    assert_eq!(result.table_moves[0].from_name(), "origin");
    assert_eq!(result.table_moves[0].to_name(), "destination");

    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains("RENAME TABLE") && sql_all.contains("DESTINATION"),
        "should produce a MySQL RENAME TABLE statement, got: {sql_all}"
    );
    assert!(
        !sql_all.contains("DROP TABLE") && !sql_all.contains("CREATE TABLE"),
        "should not fall back to CREATE+DROP, got: {sql_all}"
    );

    Ok(())
}

/// A database that no registered entity ever references (via `schema_name`)
/// must never be scanned for orphans — even without `.exclude_schema(...)`.
/// This protects a database belonging to another application/tenant sharing
/// the same MySQL server from being touched at all.
#[tokio::test]
#[cfg(feature = "sqlx-mysql")]
async fn test_unreferenced_schema_never_scanned_for_orphans() -> Result<(), DbErr> {
    let _guard = SCHEMA_LOCK.lock().await;
    let db = fresh_multi_schema_db("entity_first_mysql_multi_schema_exclude").await?;

    let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
    let base_url = match url.rfind('/') {
        Some(pos) if pos > "mysql://".len() => url[..pos].to_owned(),
        _ => url.clone(),
    };
    let root = Database::connect(ConnectOptions::new(url.clone())).await?;
    root.execute_unprepared("DROP DATABASE IF EXISTS `entity_first_mysql_foreign_schema`")
        .await?;
    root.execute_unprepared("CREATE DATABASE `entity_first_mysql_foreign_schema`")
        .await?;
    let foreign_db = Database::connect(ConnectOptions::new(format!(
        "{base_url}/entity_first_mysql_foreign_schema"
    )))
    .await?;
    foreign_db
        .execute_unprepared("CREATE TABLE foreign_table (id INT NOT NULL)")
        .await?;
    assert!(schema_exists(&db, "entity_first_mysql_foreign_schema").await?);

    // No entity references the foreign database, so it must never be scanned,
    // dangerous discovery or not.
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
        !sql_all.contains("entity_first_mysql_foreign_schema")
            && !sql_all.contains("foreign_table"),
        "a database no entity references must never be scanned, got: {sql_all}"
    );

    Ok(())
}
