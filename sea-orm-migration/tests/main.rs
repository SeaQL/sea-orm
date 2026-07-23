mod common;

use common::migrator::*;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbBackend, DbErr, Statement};
use sea_orm_migration::{MigratorTraitSelf, migrator::MigrationStatus, prelude::*};

#[tokio::test]
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
        with_self::Migrator { i: 12 },
        "sea_orm_migration_self",
        "public",
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

    run_transaction_test(url, "sea_orm_migration_txn", "public").await?;

    run_read_only_test(
        url,
        default::Migrator,
        "sea_orm_migration_read_only",
        "public",
    )
    .await?;

    Ok(())
}

/// Coverage for the read-only migration-status API (discussion #3141): querying status must
/// not run `CREATE TABLE`, so it works for a database user without DDL privileges. Runs on
/// every backend the suite is pointed at.
async fn run_read_only_test<M>(
    url: &str,
    migrator: M,
    db_name: &str,
    schema: &str,
) -> Result<(), DbErr>
where
    M: MigratorTraitSelf,
{
    let db = &create_db(url, db_name, schema).await?;
    let manager = SchemaManager::new(db);
    let table = migrator.migration_table_name().to_string();
    let total = migrator.get_migration_files().len();

    // Fresh database: the migration table does not exist yet.
    assert!(!manager.has_table(table.as_str()).await?);

    // Read-only queries report every migration as pending, WITHOUT creating the table.
    assert_eq!(
        migrator.get_pending_migrations_read_only(db).await?.len(),
        total
    );
    assert!(
        migrator
            .get_applied_migrations_read_only(db)
            .await?
            .is_empty()
    );
    assert!(
        migrator
            .get_migration_with_status_read_only(db)
            .await?
            .iter()
            .all(|m| m.status() == MigrationStatus::Pending)
    );
    assert!(!manager.has_table(table.as_str()).await?);

    // With the table present but empty (created here by the installing path), the read-only
    // queries still work and report everything pending.
    migrator.install(db).await?;
    assert!(manager.has_table(table.as_str()).await?);
    assert_eq!(
        migrator.get_pending_migrations_read_only(db).await?.len(),
        total
    );

    // After applying one migration, the read-only queries reflect the applied state.
    migrator.up(db, Some(1)).await?;
    assert_eq!(
        migrator.get_applied_migrations_read_only(db).await?.len(),
        1
    );
    assert_eq!(
        migrator.get_pending_migrations_read_only(db).await?.len(),
        total - 1
    );

    Ok(())
}

async fn create_db(
    url: &str,
    db_name: &str,
    schema: &str,
) -> Result<sea_orm::DatabaseConnection, DbErr> {
    let db_connect = |url: String| async {
        let connect_options = ConnectOptions::new(url)
            .set_schema_search_path(format!("{schema},public"))
            .to_owned();

        Database::connect(connect_options).await
    };

    let db = db_connect(url.to_owned()).await?;

    match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute_raw(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{db_name}`;"),
            ))
            .await?;

            let url = format!("{url}/{db_name}");
            db_connect(url).await
        }
        DbBackend::Postgres => {
            db.execute_raw(Statement::from_string(
                db.get_database_backend(),
                format!("DROP DATABASE IF EXISTS \"{db_name}\";"),
            ))
            .await?;
            db.execute_raw(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE \"{db_name}\";"),
            ))
            .await?;

            let url = format!("{url}/{db_name}");
            let db = db_connect(url).await?;

            db.execute_raw(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\";"),
            ))
            .await?;

            Ok(db)
        }
        DbBackend::Sqlite => Ok(db),
        db => Err(DbErr::BackendNotSupported {
            db: db.as_str(),
            ctx: "create_db",
        }),
    }
}

async fn run_migration<M>(url: &str, migrator: M, db_name: &str, schema: &str) -> Result<(), DbErr>
where
    M: MigratorTraitSelf,
{
    let db = &create_db(url, db_name, schema).await?;
    let manager = SchemaManager::new(db);

    println!("\nMigrator::status");
    migrator.status(db).await?;

    println!("\nMigrator::install");
    migrator.install(db).await?;

    let migration_table_name = migrator.migration_table_name().to_string();
    let migration_table_name = migration_table_name.as_str();
    assert!(manager.has_table(migration_table_name).await?);
    if migration_table_name != "seaql_migrations" {
        assert!(!manager.has_table("seaql_migrations").await?);
    }

    println!("\nMigrator::reset");
    migrator.reset(db).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::up");
    migrator.up(db, Some(0)).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::up");
    migrator.up(db, Some(1)).await?;

    println!("\nMigrator::get_pending_migrations");
    let migrations = migrator.get_pending_migrations(db).await?;
    assert_eq!(migrations.len(), 5);

    let migration = migrations.get(0).unwrap();
    assert_eq!(migration.name(), "m20220118_000002_create_fruit_table");
    assert_eq!(migration.status(), MigrationStatus::Pending);

    assert!(manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::down");
    migrator.down(db, Some(0)).await?;

    assert!(manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::down");
    migrator.down(db, Some(1)).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    // Tests rolling back a failing migration on Postgres.
    // With per-migration transactions, only the failing migration is rolled back;
    // earlier migrations that committed successfully are preserved.
    if matches!(db.get_database_backend(), DbBackend::Postgres) {
        println!("\nRoll back changes when encounter errors");

        // Set a flag to throw error inside `m20230109_000001_seed_cake_table.rs`
        unsafe {
            std::env::set_var("ABORT_MIGRATION", "YES");
        }

        // Should throw an error
        println!("\nMigrator::up");
        assert_eq!(
            migrator.up(db, None).await,
            Err(DbErr::Migration(
                "Abort migration and rollback changes".into()
            ))
        );

        println!("\nMigrator::status");
        migrator.status(db).await?;

        // Only the failing migration (m20230109) is rolled back;
        // earlier migrations (cake, fruit, etc.) committed successfully
        assert!(manager.has_table("cake").await?);
        assert!(manager.has_table("fruit").await?);

        // Unset the flag
        unsafe {
            std::env::remove_var("ABORT_MIGRATION");
        }
    }

    println!("\nMigrator::up");
    migrator.up(db, None).await?;

    println!("\nMigrator::get_applied_migrations");
    let migrations = migrator.get_applied_migrations(db).await?;
    assert_eq!(migrations.len(), 6);

    assert!(!manager.has_index("cake", "non_existent_index").await?);
    assert!(manager.has_index("cake", "cake_name_index").await?);

    let migration = migrations.get(0).unwrap();
    assert_eq!(migration.name(), "m20220118_000001_create_cake_table");
    assert_eq!(migration.status(), MigrationStatus::Applied);

    println!("\nMigrator::status");
    migrator.status(db).await?;

    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    assert!(manager.has_column("cake", "name").await?);
    assert!(manager.has_column("fruit", "cake_id").await?);

    // Tests rolling back a failing migration-down on Postgres.
    // With per-migration transactions, rollbacks happen one at a time in reverse.
    // Migrations 6-2 roll back and commit successfully. Migration 1 (drops cake
    // then ABORTs) fails, so its DROP is restored. But migration 2's DROP of
    // the fruit table already committed.
    if matches!(db.get_database_backend(), DbBackend::Postgres) {
        println!("\nRoll back changes when encounter errors");

        // Set a flag to throw error inside `m20220118_000001_create_cake_table.rs`
        unsafe {
            std::env::set_var("ABORT_MIGRATION", "YES");
        }

        // Should throw an error
        println!("\nMigrator::down");
        assert_eq!(
            migrator.down(db, None).await,
            Err(DbErr::Migration(
                "Abort migration and rollback changes".into()
            ))
        );

        println!("\nMigrator::status");
        migrator.status(db).await?;

        // Only migration 1's down was rolled back (cake table restored).
        // Migrations 2-6 were rolled back successfully (fruit table dropped).
        assert!(manager.has_table("cake").await?);
        assert!(!manager.has_table("fruit").await?);

        // Unset the flag
        unsafe {
            std::env::remove_var("ABORT_MIGRATION");
        }
    }

    println!("\nMigrator::down");
    migrator.down(db, None).await?;

    assert!(manager.has_table(migration_table_name).await?);
    if migration_table_name != "seaql_migrations" {
        assert!(!manager.has_table("seaql_migrations").await?);
    }

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::fresh");
    migrator.fresh(db).await?;

    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    println!("\nMigrator::refresh");
    migrator.refresh(db).await?;

    assert!(manager.has_table("cake").await?);
    assert!(manager.has_table("fruit").await?);

    println!("\nMigrator::reset");
    migrator.reset(db).await?;

    assert!(!manager.has_table("cake").await?);
    assert!(!manager.has_table("fruit").await?);

    println!("\nMigrator::status");
    migrator.status(db).await?;

    Ok(())
}

async fn run_transaction_test(url: &str, db_name: &str, schema: &str) -> Result<(), DbErr> {
    let db = &create_db(url, db_name, schema).await?;
    let backend = db.get_database_backend();
    let manager = SchemaManager::new(db);

    // use_transaction = None: Postgres wraps by default, others don't.
    // The assertion happens inside the migration's up()/down() body.
    println!("\nTransaction test: use_transaction = None");
    let m = transaction_test::Migrator {
        use_transaction: None,
        should_fail: false,
    };
    m.up(db, None).await?;
    assert!(manager.has_table("test_table").await?);
    m.down(db, None).await?;
    assert!(!manager.has_table("test_table").await?);
    m.reset(db).await.ok();

    // use_transaction = Some(true): forces transaction on every backend.
    println!("\nTransaction test: use_transaction = Some(true)");
    let m = transaction_test::Migrator {
        use_transaction: Some(true),
        should_fail: false,
    };
    m.up(db, None).await?;
    assert!(manager.has_table("test_table").await?);
    m.down(db, None).await?;
    assert!(!manager.has_table("test_table").await?);
    m.reset(db).await.ok();

    // use_transaction = Some(false): disables transaction, including on Postgres.
    println!("\nTransaction test: use_transaction = Some(false)");
    let m = transaction_test::Migrator {
        use_transaction: Some(false),
        should_fail: false,
    };
    m.up(db, None).await?;
    assert!(manager.has_table("test_table").await?);
    m.down(db, None).await?;
    assert!(!manager.has_table("test_table").await?);
    m.reset(db).await.ok();

    // Failure with transaction: DDL rolled back (except MySQL which auto-commits DDL).
    println!("\nTransaction test: failure with transaction");
    let m = transaction_test::Migrator {
        use_transaction: Some(true),
        should_fail: true,
    };
    assert!(m.up(db, None).await.is_err());
    if backend != DbBackend::MySql {
        assert!(
            !manager.has_table("test_table").await?,
            "DDL should be rolled back"
        );
    }
    m.reset(db).await.ok();

    // Failure without transaction: DDL persists.
    println!("\nTransaction test: failure without transaction");
    let m = transaction_test::Migrator {
        use_transaction: Some(false),
        should_fail: true,
    };
    assert!(m.up(db, None).await.is_err());
    assert!(manager.has_table("test_table").await?, "DDL should persist");
    db.execute_unprepared("DROP TABLE IF EXISTS test_table")
        .await?;
    m.reset(db).await.ok();

    // Manual transaction via manager.begin() / commit().
    println!("\nTransaction test: manual begin/commit");
    let m = transaction_test::ManualTxnMigrator;
    m.up(db, None).await?;
    assert!(manager.has_table("manual_txn_table").await?);
    m.down(db, None).await?;
    assert!(!manager.has_table("manual_txn_table").await?);
    m.reset(db).await.ok();

    Ok(())
}
