mod common;

use clap::Parser;
use common::migrator::*;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, DbErr, Statement};
use sea_orm_migration::cli::{run_migrate, Cli};
use sea_orm_migration::{migrator::MigrationStatus, prelude::*};

#[async_std::test]
async fn main() -> Result<(), DbErr> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let url = &std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");

    run_migration(url, default::Migrator, "sea_orm_migration", "public").await?;
    run_migration(
        url,
        default::Migrator,
        "sea_orm_migration_schema",
        "my_schema",
    )
    .await?;

    run_migration(
        url,
        override_migration_table_name::Migrator,
        "sea_orm_migration_table_name",
        "public",
    )
    .await?;
    run_migration(
        url,
        override_migration_table_name::Migrator,
        "sea_orm_migration_table_name_schema",
        "my_schema",
    )
    .await?;

    Ok(())
}

#[async_std::test]
async fn test_migration_generation_custom_dir() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let migration_dir = tmp_dir.path().to_str().unwrap();

    let cli = Cli::parse_from(&["migrate", "init", "--migration-dir", migration_dir]);

    let r = run_migrate(default::Migrator, cli).await;
    assert!(r.is_ok());

    // assert that migration dir was initialized
    let cargo_toml_path = format!("{}/Cargo.toml", migration_dir);
    assert!(std::path::Path::new(&cargo_toml_path).exists());

    let cli = Cli::parse_from(&[
        "migrate",
        "generate",
        "my_test_migration",
        "--migration-dir",
        migration_dir,
    ]);
    let r = run_migrate(default::Migrator, cli).await;
    assert!(r.is_ok());

    // assert that migration file was created
    let src_files = std::fs::read_dir(format!("{}/src", migration_dir)).unwrap();
    let migration_file = src_files.filter_map(Result::ok).find(|f| {
        f.file_name()
            .to_str()
            .unwrap()
            .ends_with("my_test_migration.rs")
    });
    assert!(migration_file.is_some());
}

#[async_std::test]
async fn test_init_migration_default_dir() {
    async fn _run_migrate(dir: &str, cmd: &[&str]) {
        let curr_dir = std::env::current_dir();
        std::env::set_current_dir(&dir).unwrap();

        let cli = Cli::parse_from(cmd);
        let r = run_migrate(default::Migrator, cli).await;

        // avoid side effects in tests
        if let Ok(d) = curr_dir {
            std::env::set_current_dir(d).unwrap();
        }

        assert!(r.is_ok());
    }

    let tmp_dir = tempfile::tempdir().unwrap();
    let migration_dir = tmp_dir.path().to_str().unwrap();

    _run_migrate(migration_dir, &["migrate", "init"]).await;
    // assert that migration dir was initialized
    let cargo_toml_path = format!("{}/Cargo.toml", migration_dir);
    assert!(std::path::Path::new(&cargo_toml_path).exists());

    _run_migrate(migration_dir, &["migrate", "generate", "my_test_migration"]).await;

    // assert that migration file was created
    let src_files = std::fs::read_dir(format!("{}/src", migration_dir)).unwrap();
    let migration_file = src_files.filter_map(Result::ok).find(|f| {
        f.file_name()
            .to_str()
            .unwrap()
            .ends_with("my_test_migration.rs")
    });
    assert!(migration_file.is_some());
}

async fn run_migration<Migrator>(
    url: &str,
    _: Migrator,
    db_name: &str,
    schema: &str,
) -> Result<(), DbErr>
where
    Migrator: MigratorTrait,
{
    let db_connect = |url: String| async {
        let connect_options = ConnectOptions::new(url)
            .set_schema_search_path(format!("{schema},public"))
            .to_owned();

        Database::connect(connect_options).await
    };

    let db = db_connect(url.to_owned()).await?;

    let db = &match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{db_name}`;"),
            ))
            .await?;

            let url = format!("{url}/{db_name}");
            db_connect(url).await?
        }
        DbBackend::Postgres => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{db_name}\";"),
            ))
            .await?;
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{db_name}\";"),
            ))
            .await?;

            let url = format!("{url}/{db_name}");
            let db = db_connect(url).await?;

            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\";"),
            ))
            .await?;

            db
        }
        DbBackend::Sqlite => db,
    };
    let manager = SchemaManager::new(db);

    println!("\nMigrator::status");
    Migrator::status(db).await?;

    println!("\nMigrator::install");
    Migrator::install(db).await?;

    let migration_table_name = Migrator::migration_table_name().to_string();
    let migration_table_name = migration_table_name.as_str();
    assert!(manager.has_table(migration_table_name).await?);
    if migration_table_name != "seaql_migrations" {
        assert!(!manager.has_table("seaql_migrations").await?);
    }

    println!("\nMigrator::reset");
    Migrator::reset(db).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::up");
    Migrator::up(db, Some(0)).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::up");
    Migrator::up(db, Some(1)).await?;

    println!("\nMigrator::get_pending_migrations");
    let migrations = Migrator::get_pending_migrations(db).await?;
    assert_eq!(migrations.len(), 5);

    let migration = migrations.get(0).unwrap();
    assert_eq!(migration.name(), "m20220118_000002_create_fruit_table");
    assert_eq!(migration.status(), MigrationStatus::Pending);

    assert!(manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::down");
    Migrator::down(db, Some(0)).await?;

    assert!(manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::down");
    Migrator::down(db, Some(1)).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    // Tests rolling back changes of "migrate up" when running migration on Postgres
    if matches!(db.get_database_backend(), DbBackend::Postgres) {
        println!("\nRoll back changes when encounter errors");

        // Set a flag to throw error inside `m20230109_000001_seed_cake_table.rs`
        std::env::set_var("ABORT_MIGRATION", "YES");

        // Should throw an error
        println!("\nMigrator::up");
        assert_eq!(
            Migrator::up(db, None).await,
            Err(DbErr::Migration(
                "Abort migration and rollback changes".into()
            ))
        );

        println!("\nMigrator::status");
        Migrator::status(db).await?;

        // Check migrations have been rolled back
        assert!(!manager.has_table("cake").await?);
        assert!(!manager.has_table("fruit").await?);

        // Unset the flag
        std::env::remove_var("ABORT_MIGRATION");
    }

    println!("\nMigrator::up");
    Migrator::up(db, None).await?;

    println!("\nMigrator::get_applied_migrations");
    let migrations = Migrator::get_applied_migrations(db).await?;
    assert_eq!(migrations.len(), 6);

    assert!(!manager.has_index("cake", "non_existent_index").await?);
    assert!(manager.has_index("cake", "cake_name_index").await?);

    let migration = migrations.get(0).unwrap();
    assert_eq!(migration.name(), "m20220118_000001_create_cake_table");
    assert_eq!(migration.status(), MigrationStatus::Applied);

    println!("\nMigrator::status");
    Migrator::status(db).await?;

    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    assert!(manager.has_column("cake", "name").await?);
    assert!(manager.has_column("fruit", "cake_id").await?);

    // Tests rolling back changes of "migrate down" when running migration on Postgres
    if matches!(db.get_database_backend(), DbBackend::Postgres) {
        println!("\nRoll back changes when encounter errors");

        // Set a flag to throw error inside `m20230109_000001_seed_cake_table.rs`
        std::env::set_var("ABORT_MIGRATION", "YES");

        // Should throw an error
        println!("\nMigrator::down");
        assert_eq!(
            Migrator::down(db, None).await,
            Err(DbErr::Migration(
                "Abort migration and rollback changes".into()
            ))
        );

        println!("\nMigrator::status");
        Migrator::status(db).await?;

        // Check migrations have been rolled back
        assert!(manager.has_table("cake").await?);
        assert!(manager.has_table("fruit").await?);

        // Unset the flag
        std::env::remove_var("ABORT_MIGRATION");
    }

    println!("\nMigrator::down");
    Migrator::down(db, None).await?;

    assert!(manager.has_table(migration_table_name).await?);
    if migration_table_name != "seaql_migrations" {
        assert!(!manager.has_table("seaql_migrations").await?);
    }

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::fresh");
    Migrator::fresh(db).await?;

    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    println!("\nMigrator::refresh");
    Migrator::refresh(db).await?;

    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    println!("\nMigrator::reset");
    Migrator::reset(db).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::status");
    Migrator::status(db).await?;

    Ok(())
}
