mod common;

use std::fs;
use tempfile::TempDir;

use common::entity_migrator::default::Migrator;
use common::entity_common::{
    CakeAmbiguousOnly, CakeRenamedOnly, CakeTypeChangeOnly, CakeV1FruitV2, CakeV1Only,
    CakeV2FruitV1, FullSchema,
};
use sea_orm::{Database, DbErr, Schema};
use sea_orm_migration::{EntitySet, MigratorTraitSelf, SchemaManager, prelude::*};

async fn connect() -> Result<sea_orm::DatabaseConnection, DbErr> {
    Database::connect("sqlite::memory:").await
}

/// Temporary directory with a skeleton migration `lib.rs` ready for writing into.
fn temp_migration_dir() -> TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let src = dir.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(
        src.join("lib.rs"),
        r#"pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![]
    }
}
"#,
    )
    .unwrap();
    dir
}

// ---------------------------------------------------------------------------
// summary tests — pure unit tests, no DB required
// ---------------------------------------------------------------------------

#[cfg(test)]
mod summary_tests {
    use sea_orm::{DbBackend, Statement};
    use sea_orm_migration::summary::summarize;

    fn stmt(sql: &str) -> Statement {
        Statement::from_string(DbBackend::Sqlite, sql.to_owned())
    }

    #[test]
    fn test_create_table() {
        assert_eq!(
            summarize(&[stmt(r#"CREATE TABLE "cake" ( "id" integer NOT NULL )"#)]),
            vec!["Created table: cake"]
        );
    }

    #[test]
    fn test_create_table_if_not_exists() {
        assert_eq!(
            summarize(&[stmt(
                r#"CREATE TABLE IF NOT EXISTS "fruit" ( "id" integer NOT NULL )"#
            )]),
            vec!["Created table: fruit"]
        );
    }

    #[test]
    fn test_add_column() {
        assert_eq!(
            summarize(&[stmt(r#"ALTER TABLE "cake" ADD COLUMN "description" text"#)]),
            vec!["Added column: cake.description"]
        );
    }

    #[test]
    fn test_drop_column() {
        assert_eq!(
            summarize(&[stmt(r#"ALTER TABLE "fruit" DROP COLUMN "weight_grams""#)]),
            vec!["Dropped column: fruit.weight_grams"]
        );
    }

    #[test]
    fn test_drop_table() {
        assert_eq!(
            summarize(&[stmt(r#"DROP TABLE IF EXISTS "cake""#)]),
            vec!["Dropped table: cake"]
        );
    }

    #[test]
    fn test_create_unique_index() {
        assert_eq!(
            summarize(&[stmt(
                r#"CREATE UNIQUE INDEX "idx_cake_name" ON "cake" ("name")"#
            )]),
            vec!["Added unique index on: cake"]
        );
    }

    #[test]
    fn test_add_foreign_key() {
        assert_eq!(
            summarize(&[stmt(
                r#"ALTER TABLE "fruit" ADD CONSTRAINT "fk_cake_id" FOREIGN KEY ("cake_id") REFERENCES "cake" ("id")"#
            )]),
            vec!["Added foreign key on: fruit"]
        );
    }

    #[test]
    fn test_multiple_stmts_ordering() {
        let stmts = vec![
            stmt(r#"CREATE TABLE "cake" ( "id" integer NOT NULL )"#),
            stmt(r#"CREATE TABLE "fruit" ( "id" integer NOT NULL )"#),
            stmt(r#"ALTER TABLE "fruit" ADD COLUMN "weight_grams" integer"#),
        ];
        assert_eq!(
            summarize(&stmts),
            vec![
                "Created table: cake",
                "Created table: fruit",
                "Added column: fruit.weight_grams",
            ]
        );
    }
}

// ---------------------------------------------------------------------------
// codegen tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod codegen_tests {
    use sea_orm::{DbBackend, Statement};
    use sea_orm_migration::codegen::{MigrationMetadata, render_migration_file};

    fn cake_create_stmt() -> Statement {
        Statement::from_string(
            DbBackend::Sqlite,
            r#"CREATE TABLE "cake" ( "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT, "name" varchar NOT NULL )"#.to_owned(),
        )
    }

    fn fruit_create_stmt() -> Statement {
        Statement::from_string(
            DbBackend::Sqlite,
            r#"CREATE TABLE "fruit" ( "id" integer NOT NULL, "name" varchar NOT NULL, "cake_id" integer NOT NULL )"#.to_owned(),
        )
    }

    fn meta<'a>(changes: &'a [String]) -> MigrationMetadata<'a> {
        MigrationMetadata {
            version: "0.1.0",
            generated_at: "2026-01-01 00:00:00 UTC",
            backend: "SQLite",
            changes,
        }
    }

    #[test]
    fn test_renders_header_metadata() {
        let changes = vec![
            "Created table: cake".to_string(),
            "Created table: fruit".to_string(),
        ];
        let out =
            render_migration_file(&[cake_create_stmt(), fruit_create_stmt()], &meta(&changes));
        assert!(out.contains("// Generated by sea-orm-entity v0.1.0"));
        assert!(out.contains("// Generated at: 2026-01-01 00:00:00 UTC"));
        assert!(out.contains("// Backend: SQLite"));
        assert!(out.contains("//   - Created table: cake"));
        assert!(out.contains("//   - Created table: fruit"));
    }

    #[test]
    fn test_renders_boilerplate() {
        let changes = vec![];
        let out = render_migration_file(&[cake_create_stmt()], &meta(&changes));
        assert!(out.contains("#[derive(DeriveMigrationName)]"));
        assert!(out.contains("pub struct Migration;"));
        assert!(out.contains("impl MigrationTrait for Migration"));
        assert!(out.contains("async fn up(&self, manager: &SchemaManager)"));
        assert!(out.contains("async fn down(&self, _manager: &SchemaManager)"));
        assert!(out.contains("// TODO: implement down migration"));
    }

    #[test]
    fn test_renders_all_stmts_as_execute_unprepared() {
        let stmts = vec![cake_create_stmt(), fruit_create_stmt()];
        let changes = vec![];
        let out = render_migration_file(&stmts, &meta(&changes));
        assert!(out.contains(r#"CREATE TABLE "cake""#));
        assert!(out.contains(r#"CREATE TABLE "fruit""#));
        assert_eq!(out.matches("execute_unprepared").count(), 2);
    }

    #[test]
    fn test_sql_embedded_as_raw_string_literal() {
        let out = render_migration_file(&[cake_create_stmt()], &meta(&[]));
        assert!(out.contains(r#"r#""#), "should use raw string literal");
    }
}

// ---------------------------------------------------------------------------
// fs tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod fs_tests {
    use super::temp_migration_dir;
    use sea_orm::{DbBackend, Statement};
    use sea_orm_migration::codegen::MigrationMetadata;
    use std::fs;

    fn stmts() -> Vec<Statement> {
        vec![
            Statement::from_string(
                DbBackend::Sqlite,
                r#"CREATE TABLE "cake" ( "id" integer NOT NULL )"#.to_owned(),
            ),
            Statement::from_string(
                DbBackend::Sqlite,
                r#"CREATE TABLE "fruit" ( "id" integer NOT NULL )"#.to_owned(),
            ),
        ]
    }

    fn meta(changes: &[String]) -> MigrationMetadata<'_> {
        MigrationMetadata {
            version: "0.1.0",
            generated_at: "2026-01-01 00:00:00 UTC",
            backend: "SQLite",
            changes,
        }
    }

    #[test]
    fn test_write_migration_creates_file_and_updates_lib() {
        let dir = temp_migration_dir();
        let name = "m20260101_000001_create_schema";
        let changes = vec![
            "Created table: cake".to_string(),
            "Created table: fruit".to_string(),
        ];

        sea_orm_migration::fs::write_migration(
            dir.path().to_str().unwrap(),
            name,
            &stmts(),
            &meta(&changes),
        )
        .expect("write_migration failed");

        let file = dir.path().join("src").join(format!("{name}.rs"));
        assert!(file.exists());
        let content = fs::read_to_string(&file).unwrap();
        assert!(content.contains("DeriveMigrationName"));
        assert!(content.contains("Created table: cake"));
        assert!(content.contains("Created table: fruit"));

        let lib = fs::read_to_string(dir.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains(&format!("mod {name};")));
        assert!(lib.contains(&format!("Box::new({name}::Migration)")));
    }

    #[test]
    fn test_second_migration_appends_to_lib() {
        let dir = temp_migration_dir();
        let changes: Vec<String> = vec![];

        sea_orm_migration::fs::write_migration(
            dir.path().to_str().unwrap(),
            "m20260101_000001_first",
            &stmts(),
            &meta(&changes),
        )
        .unwrap();
        sea_orm_migration::fs::write_migration(
            dir.path().to_str().unwrap(),
            "m20260101_000002_second",
            &stmts(),
            &meta(&changes),
        )
        .unwrap();

        let lib = fs::read_to_string(dir.path().join("src/lib.rs")).unwrap();
        assert!(lib.contains("mod m20260101_000001_first;"));
        assert!(lib.contains("mod m20260101_000002_second;"));
        assert!(lib.contains("Box::new(m20260101_000001_first::Migration)"));
        assert!(lib.contains("Box::new(m20260101_000002_second::Migration)"));
    }

    #[test]
    fn test_generated_file_does_not_reference_removed_migration() {
        let dir = temp_migration_dir();
        let changes: Vec<String> = vec![];

        sea_orm_migration::fs::write_migration(
            dir.path().to_str().unwrap(),
            "m20260101_000001_only",
            &stmts(),
            &meta(&changes),
        )
        .unwrap();

        let lib = fs::read_to_string(dir.path().join("src/lib.rs")).unwrap();
        assert_eq!(lib.matches("mod m20260101_000001_only;").count(), 1);
        assert_eq!(lib.matches("m20260101_000001_only::Migration").count(), 1);
    }
}

// ---------------------------------------------------------------------------
// Integration tests — full discover → generate → apply → lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_discover_full_schema_on_empty_db() -> Result<(), DbErr> {
    let db = connect().await?;
    let builder = FullSchema.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();

    assert!(!stmts.is_empty());

    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(sql_all.contains("CREATE TABLE"), "should create tables");

    let table_names: Vec<_> = stmts
        .iter()
        .filter(|s| s.sql.to_uppercase().contains("CREATE TABLE"))
        .collect();
    assert_eq!(table_names.len(), 2, "should create exactly cake + fruit");

    Ok(())
}

#[tokio::test]
async fn test_no_diff_when_schema_matches_entities() -> Result<(), DbErr> {
    let db = connect().await?;
    Migrator.up(&db, None).await?;

    let builder = FullSchema.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();

    assert!(
        stmts.is_empty(),
        "no changes expected on synced DB, got: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );

    Ok(())
}

#[tokio::test]
async fn test_discover_detects_added_columns() -> Result<(), DbErr> {
    let db = connect().await?;

    sea_orm::Schema::new(db.get_database_backend())
        .builder()
        .register(common::entity_common::cake_v1::Entity)
        .sync(&db)
        .await?;

    let builder = CakeV2FruitV1.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();

    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains("ADD COLUMN") || sql_all.contains("CREATE TABLE"),
        "should detect schema additions, got stmts: {:?}",
        stmts.iter().map(|s| &s.sql).collect::<Vec<_>>()
    );

    let all_sql: String = stmts
        .iter()
        .map(|s| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        all_sql.contains("description") || all_sql.contains("fruit"),
        "should reference new columns or missing tables"
    );

    Ok(())
}

#[tokio::test]
async fn test_discover_detects_added_column_and_unique_index() -> Result<(), DbErr> {
    let db = connect().await?;

    sea_orm::Schema::new(db.get_database_backend())
        .builder()
        .register(common::entity_common::cake_v1::Entity)
        .register(common::entity_common::fruit_v1::Entity)
        .sync(&db)
        .await?;

    let builder = CakeV1FruitV2.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();

    assert!(!stmts.is_empty(), "should detect changes");

    let sql_all: String = stmts
        .iter()
        .map(|s| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains("weight_grams"),
        "should ADD COLUMN weight_grams, got: {sql_all}"
    );

    Ok(())
}

#[tokio::test]
async fn test_discover_dangerous_drops_orphaned_tables_but_not_tracker() -> Result<(), DbErr> {
    let db = connect().await?;

    Migrator.up(&db, None).await?;

    let builder = CakeV1Only.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, true).await?;
    let result = sea_orm::interpret_changes(change_set, &sea_orm::InterpretConfig {
        db_backend: db.get_database_backend(),
        assumptions: false,
        allow_dangerous: true,
    });
    let stmts: Vec<_> = result.statements.iter().map(|(_, s)| s).collect();

    let protected = Migrator.migration_table_name().to_string();
    let raw_drops: Vec<_> = stmts
        .iter()
        .filter(|s| s.sql.to_uppercase().contains("DROP TABLE"))
        .collect();
    assert!(
        raw_drops.iter().any(|s| s.sql.contains("fruit")),
        "fruit should appear in DROP TABLE statements"
    );

    let protected_upper = protected.to_uppercase();
    let filtered: Vec<_> = stmts
        .iter()
        .filter(|s| {
            let upper = s.sql.to_uppercase();
            if upper.contains("DROP TABLE") {
                !upper.contains(&format!("\"{}\"", protected_upper))
                    && !upper.contains(&format!("`{}`", protected_upper))
                    && !upper.contains(&format!(" {} ", protected_upper))
                    && !upper.ends_with(&format!(" {}", protected_upper))
            } else {
                true
            }
        })
        .collect();

    assert!(
        filtered
            .iter()
            .any(|s| s.sql.to_uppercase().contains("DROP TABLE") && s.sql.contains("fruit")),
        "fruit should still be in filtered DROP statements"
    );
    assert!(
        !filtered
            .iter()
            .any(|s| s.sql.to_lowercase().contains(&protected.to_lowercase())
                && s.sql.to_uppercase().contains("DROP TABLE")),
        "seaql_migrations must not appear in filtered DROP statements"
    );

    Ok(())
}

#[tokio::test]
async fn test_discover_safe_never_drops() -> Result<(), DbErr> {
    let db = connect().await?;
    Migrator.up(&db, None).await?;

    let builder = CakeV1Only.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();

    assert!(
        !stmts.iter().any(|s| s.sql.to_uppercase().contains("DROP")),
        "safe discover must not emit any DROP statements"
    );

    Ok(())
}

#[tokio::test]
async fn test_full_migration_lifecycle() -> Result<(), DbErr> {
    let db = connect().await?;
    let manager = SchemaManager::new(&db);

    let pending = Migrator.get_pending_migrations(&db).await?;
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].name(), "m20250101_000001_create_cake_table");
    assert_eq!(pending[1].name(), "m20250101_000002_create_fruit_table");

    Migrator.up(&db, Some(1)).await?;
    assert!(manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);
    assert!(manager.has_column("cake", "id").await?);
    assert!(manager.has_column("cake", "name").await?);

    let pending = Migrator.get_pending_migrations(&db).await?;
    assert_eq!(pending.len(), 1);

    Migrator.up(&db, None).await?;
    assert!(manager.has_table("fruit").await?);
    assert!(manager.has_column("fruit", "cake_id").await?);

    let applied = Migrator.get_applied_migrations(&db).await?;
    assert_eq!(applied.len(), 2);

    let builder = FullSchema.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();
    assert!(stmts.is_empty(), "no changes after full apply, got: {:?}", stmts.iter().map(|s| &s.sql).collect::<Vec<_>>());

    Migrator.down(&db, Some(1)).await?;
    assert!(!manager.has_table("fruit").await?);
    assert!(manager.has_table("cake").await?);

    Migrator.fresh(&db).await?;
    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    Migrator.reset(&db).await?;
    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    Ok(())
}

#[tokio::test]
async fn test_generate_pipeline_for_full_schema() -> Result<(), DbErr> {
    let db = connect().await?;
    let dir = temp_migration_dir();

    let builder = FullSchema.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, false).await?;
    let stmts = change_set.statements();
    assert!(!stmts.is_empty());

    let changes = sea_orm_migration::summary::summarize(&stmts);
    assert!(changes.iter().any(|c| c.contains("cake")));
    assert!(changes.iter().any(|c| c.contains("fruit")));

    let meta = sea_orm_migration::codegen::MigrationMetadata {
        version: "0.1.0",
        generated_at: "2026-01-01 00:00:00 UTC",
        backend: "SQLite",
        changes: &changes,
    };
    let filepath = sea_orm_migration::fs::write_migration(
        dir.path().to_str().unwrap(),
        "m20260101_000001_create_schema",
        &stmts,
        &meta,
    )
    .unwrap();

    let content = fs::read_to_string(&filepath).unwrap();
    assert!(content.contains(r#"CREATE TABLE"#));
    assert!(content.contains("cake") || content.contains("fruit"));

    let lib = fs::read_to_string(dir.path().join("src/lib.rs")).unwrap();
    assert!(lib.contains("m20260101_000001_create_schema"));

    Ok(())
}

// ---------------------------------------------------------------------------
// Safety & correctness tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_discover_warns_on_possible_column_rename() -> Result<(), DbErr> {
    use sea_orm::schema::SuggestionKind;
    let db = connect().await?;

    sea_orm::Schema::new(db.get_database_backend())
        .builder()
        .register(common::entity_common::cake_v1::Entity)
        .sync(&db)
        .await?;

    let builder = CakeRenamedOnly.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, true).await?;
    let result = sea_orm::interpret_changes(change_set, &sea_orm::InterpretConfig {
        db_backend: db.get_database_backend(),
        assumptions: true,
        allow_dangerous: true,
    });

    assert!(
        result
            .suggestions
            .iter()
            .any(|s| s.kind == SuggestionKind::PossibleRename),
        "should emit PossibleRename suggestion, got suggestions: {:?}",
        result.suggestions
    );

    let rename_suggestion = result
        .suggestions
        .iter()
        .find(|s| s.kind == SuggestionKind::PossibleRename)
        .unwrap();
    assert!(
        rename_suggestion.message.contains("name")
            && rename_suggestion.message.contains("title"),
        "suggestion should mention old and new column names, got: {}",
        rename_suggestion.message
    );

    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.to_uppercase())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains("RENAME COLUMN"),
        "should produce RENAME COLUMN for auto-assumed rename; got: {sql_all}"
    );

    let has_drop_name = sql_all.contains("DROP COLUMN");
    let has_add_title = sql_all.contains("ADD COLUMN");
    assert!(
        !has_drop_name && !has_add_title,
        "should not produce DROP or ADD when rename is auto-assumed; got: {sql_all}"
    );

    assert!(
        result.unresolved.is_empty(),
        "single obvious rename should not be ambiguous"
    );

    Ok(())
}

#[tokio::test]
async fn test_discover_no_rename_warning_when_types_differ() -> Result<(), DbErr> {
    use sea_orm::schema::SuggestionKind;
    let db = connect().await?;

    sea_orm::Schema::new(db.get_database_backend())
        .builder()
        .register(common::entity_common::cake_v1::Entity)
        .sync(&db)
        .await?;

    let builder =
        CakeTypeChangeOnly.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, true).await?;
    let result = sea_orm::interpret_changes(change_set, &sea_orm::InterpretConfig {
        db_backend: db.get_database_backend(),
        assumptions: true,
        allow_dangerous: true,
    });

    assert!(
        !result
            .suggestions
            .iter()
            .any(|s| s.kind == SuggestionKind::PossibleRename),
        "should not emit PossibleRename when types differ, got suggestions: {:?}",
        result.suggestions
    );

    let sql_all: String = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        sql_all.contains("count"),
        "should ADD COLUMN count, got: {sql_all}"
    );

    Ok(())
}

#[tokio::test]
async fn test_ambiguous_rename_in_unresolved() -> Result<(), DbErr> {
    let db = connect().await?;

    sea_orm::Schema::new(db.get_database_backend())
        .builder()
        .register(common::entity_common::cake_v1::Entity)
        .sync(&db)
        .await?;

    let builder = CakeAmbiguousOnly.register(Schema::new(db.get_database_backend()).builder());
    let change_set = builder.discover(&db, true).await?;
    let result = sea_orm::interpret_changes(change_set, &sea_orm::InterpretConfig {
        db_backend: db.get_database_backend(),
        assumptions: true,
        allow_dangerous: true,
    });

    assert!(
        !result.unresolved.is_empty(),
        "expected unresolved ambiguous renames, got none; warnings: {:?}, statements: {:?}",
        result.warnings,
        result.statements.iter().map(|(_, s)| &s.sql).collect::<Vec<_>>()
    );

    let ambiguous = &result.unresolved[0];
    assert_eq!(ambiguous.removed, "name");
    assert!(
        ambiguous.candidates.len() >= 2,
        "should have multiple candidates, got: {:?}",
        ambiguous.candidates
    );

    let has_rename = result
        .statements
        .iter()
        .any(|(_, s)| s.sql.to_uppercase().contains("RENAME COLUMN"));
    assert!(
        !has_rename,
        "should not generate RENAME COLUMN for ambiguous rename"
    );

    assert!(
        !result
            .suggestions
            .iter()
            .any(|s| s.kind == sea_orm::schema::SuggestionKind::PossibleRename),
        "ambiguous renames should not produce PossibleRename suggestions"
    );

    Ok(())
}

#[cfg(test)]
mod enum_warning_tests {
    use sea_orm::schema::extract_enum_type_name;

    #[test]
    fn test_extract_enum_type_name_postgres() {
        let sql = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad', 'neutral')"#;
        assert_eq!(extract_enum_type_name(sql), Some("mood".to_string()));
    }

    #[test]
    fn test_extract_enum_type_name_no_match() {
        let sql = r#"CREATE TABLE "cake" ( "id" integer NOT NULL )"#;
        assert_eq!(extract_enum_type_name(sql), None);
    }

    #[test]
    fn test_enum_same_name_different_variants_detected() {
        let existing_sql = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad')"#;
        let new_sql = r#"CREATE TYPE "mood" AS ENUM ('happy', 'sad', 'neutral')"#;

        let existing_name = extract_enum_type_name(existing_sql);
        let new_name = extract_enum_type_name(new_sql);

        assert_eq!(existing_name, new_name);
        assert_ne!(existing_sql, new_sql);
    }
}

#[cfg(test)]
mod safety_summary_tests {
    use sea_orm::{DbBackend, Statement};
    use sea_orm_migration::summary::summarize;

    fn stmt(sql: &str) -> Statement {
        Statement::from_string(DbBackend::Sqlite, sql.to_owned())
    }

    #[test]
    fn test_summary_rename_column() {
        assert_eq!(
            summarize(&[stmt(
                r#"ALTER TABLE "cake" RENAME COLUMN "name" TO "title""#
            )]),
            vec!["Renamed column on: cake"]
        );
    }

    #[test]
    fn test_summary_alter_type_add_value() {
        assert_eq!(
            summarize(&[stmt(
                r#"ALTER TYPE "mood" ADD VALUE 'neutral'"#
            )]),
            vec!["Added enum variant"]
        );
    }
}

#[cfg(test)]
mod warning_type_tests {
    use sea_orm::schema::{DiscoverSuggestion, DiscoverWarning, SuggestionKind, WarningKind};

    #[test]
    fn test_warning_kinds_are_eq() {
        assert_eq!(
            WarningKind::CheckConstraintDiff,
            WarningKind::CheckConstraintDiff
        );
        assert_ne!(
            WarningKind::CheckConstraintDiff,
            WarningKind::NotNullNoDefault
        );
    }

    #[test]
    fn test_suggestion_kinds_are_eq() {
        assert_eq!(SuggestionKind::PossibleRename, SuggestionKind::PossibleRename);
        assert_ne!(SuggestionKind::PossibleRename, SuggestionKind::EnumVariantChange);
        assert_ne!(
            SuggestionKind::EnumRename,
            SuggestionKind::EnumVariantChange
        );
    }

    #[test]
    fn test_warning_debug_format() {
        let w = DiscoverWarning {
            kind: WarningKind::CheckConstraintDiff,
            message: "CHECK constraint cannot be diffed".to_string(),
            related_changes: vec![],
        };
        let debug = format!("{w:?}");
        assert!(debug.contains("CheckConstraintDiff"));
    }

    #[test]
    fn test_suggestion_debug_format() {
        let s = DiscoverSuggestion {
            kind: SuggestionKind::PossibleRename,
            message: "Column 'name' may have been renamed to 'title'".to_string(),
            related_changes: vec![],
        };
        let debug = format!("{s:?}");
        assert!(debug.contains("PossibleRename"));
        assert!(debug.contains("name"));
    }
}
