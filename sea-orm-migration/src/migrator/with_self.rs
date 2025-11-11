use super::{Migration, MigrationStatus, exec::*};
use crate::{IntoSchemaManagerConnection, MigrationTrait, SchemaManager, seaql_migrations};
use sea_orm::sea_query::IntoIden;
use sea_orm::{ConnectionTrait, DbBackend, DbErr, DynIden, TransactionTrait};

use tracing::info;

/// Performing migrations on a database
#[async_trait::async_trait]
pub trait MigratorTraitSelf: Sized + Send + Sync {
    /// Vector of migrations in time sequence
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;

    /// Name of the migration table, it is `seaql_migrations` by default
    fn migration_table_name(&self) -> DynIden {
        seaql_migrations::Entity.into_iden()
    }

    /// Get list of migrations wrapped in `Migration` struct
    fn get_migration_files(&self) -> Vec<Migration> {
        self.migrations()
            .into_iter()
            .map(|migration| Migration {
                migration,
                status: MigrationStatus::Pending,
            })
            .collect()
    }

    /// Get list of applied migrations from database
    async fn get_migration_models<C>(&self, db: &C) -> Result<Vec<seaql_migrations::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;
        get_migration_models(db, self.migration_table_name()).await
    }

    /// Get list of migrations with status
    async fn get_migration_with_status<C>(&self, db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;
        get_migration_with_status(
            self.get_migration_files(),
            self.get_migration_models(db).await?,
        )
    }

    /// Get list of pending migrations
    async fn get_pending_migrations<C>(&self, db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;
        Ok(self
            .get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Pending)
            .collect())
    }

    /// Get list of applied migrations
    async fn get_applied_migrations<C>(&self, db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;
        Ok(self
            .get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Applied)
            .collect())
    }

    /// Create migration table `seaql_migrations` in the database
    async fn install<C>(&self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        install(db, self.migration_table_name()).await
    }

    /// Check the status of all migrations
    async fn status<C>(&self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;

        info!("Checking migration status");

        for Migration { migration, status } in self.get_migration_with_status(db).await? {
            info!("Migration '{}'... {}", migration.name(), status);
        }

        Ok(())
    }

    /// Drop all tables from the database, then reapply all migrations
    async fn fresh<'c, C>(&self, db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        exec_with_connection!(db, async |manager| { exec_fresh(self, manager).await }).await
    }

    /// Rollback all applied migrations, then reapply all migrations
    async fn refresh<'c, C>(&self, db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        exec_with_connection!(db, async |manager| {
            exec_down(self, manager, None).await?;
            exec_up(self, manager, None).await
        })
        .await
    }

    /// Rollback all applied migrations
    async fn reset<'c, C>(&self, db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        exec_with_connection!(db, async |manager| {
            // Rollback all applied migrations first
            exec_down(self, manager, None).await?;

            // Then drop the migration table itself
            uninstall(manager, self.migration_table_name()).await
        })
        .await
    }

    /// Uninstall migration tracking table only (non-destructive)
    /// This will drop the `seaql_migrations` table but won't rollback other schema changes.
    async fn uninstall<'c, C>(&self, db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        exec_with_connection!(db, async |manager| {
            uninstall(manager, self.migration_table_name()).await
        })
        .await
    }

    /// Apply pending migrations
    async fn up<'c, C>(&self, db: C, steps: Option<u32>) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        exec_with_connection!(db, async |manager| { exec_up(self, manager, steps).await }).await
    }

    /// Rollback applied migrations
    async fn down<'c, C>(&self, db: C, steps: Option<u32>) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        exec_with_connection!(db, async |manager| {
            exec_down(self, manager, steps).await
        })
        .await
    }
}

async fn exec_fresh<M>(migrator: &M, manager: &SchemaManager<'_>) -> Result<(), DbErr>
where
    M: MigratorTraitSelf,
{
    let db = manager.get_connection();

    migrator.install(db).await?;

    drop_everything(db).await?;

    exec_up(migrator, manager, None).await
}

async fn exec_up<M>(
    migrator: &M,
    manager: &SchemaManager<'_>,
    steps: Option<u32>,
) -> Result<(), DbErr>
where
    M: MigratorTraitSelf,
{
    let db = manager.get_connection();

    migrator.install(db).await?;

    exec_up_with(
        manager,
        steps,
        migrator.get_pending_migrations(db).await?,
        migrator.migration_table_name(),
    )
    .await
}

async fn exec_down<M>(
    migrator: &M,
    manager: &SchemaManager<'_>,
    steps: Option<u32>,
) -> Result<(), DbErr>
where
    M: MigratorTraitSelf,
{
    let db = manager.get_connection();

    migrator.install(db).await?;

    exec_down_with(
        manager,
        steps,
        migrator.get_applied_migrations(db).await?,
        migrator.migration_table_name(),
    )
    .await
}
