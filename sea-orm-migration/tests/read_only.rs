//! Self-contained (in-memory SQLite) coverage for the read-only migration-status API added
//! for discussion #3141: querying migration status must not create the `seaql_migrations`
//! table, so a database user without DDL privileges can use it.
//!
//! Run: cargo test --features sqlx-sqlite,runtime-tokio-rustls --test read_only

use sea_orm::Database;
use sea_orm_migration::{MigratorTraitSelf, async_trait, migrator::MigrationStatus, prelude::*};

struct M1;
impl MigrationName for M1 {
    fn name(&self) -> &str {
        "m000001_first"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for M1 {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct M2;
impl MigrationName for M2 {
    fn name(&self) -> &str {
        "m000002_second"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for M2 {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

struct Migrator;
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(M1), Box::new(M2)]
    }
}

#[tokio::test]
async fn read_only_status_does_not_create_table() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let manager = SchemaManager::new(&db);

    // On a fresh database the read-only queries report every migration as pending...
    // (`Migrator` implements both `MigratorTrait` and `MigratorTraitSelf`, so the static
    // calls are fully qualified to disambiguate — the same applies to existing methods.)
    let pending = <Migrator as MigratorTrait>::get_pending_migrations_read_only(&db).await?;
    assert_eq!(pending.len(), 2);
    assert!(
        <Migrator as MigratorTrait>::get_applied_migrations_read_only(&db)
            .await?
            .is_empty()
    );
    let all = <Migrator as MigratorTrait>::get_migration_with_status_read_only(&db).await?;
    assert_eq!(all.len(), 2);
    assert!(all.iter().all(|m| m.status() == MigrationStatus::Pending));

    // ...WITHOUT creating the migration table (the whole point for read-only DB users).
    assert!(!manager.has_table("seaql_migrations").await?);

    // The `with-self` migrator exposes the same read-only API and is likewise non-creating.
    assert_eq!(
        Migrator.get_pending_migrations_read_only(&db).await?.len(),
        2
    );
    assert!(!manager.has_table("seaql_migrations").await?);

    // Contrast: the ordinary (installing) variant DOES create the table.
    assert_eq!(
        <Migrator as MigratorTrait>::get_pending_migrations(&db)
            .await?
            .len(),
        2
    );
    assert!(manager.has_table("seaql_migrations").await?);

    Ok(())
}
