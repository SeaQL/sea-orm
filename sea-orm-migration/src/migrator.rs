mod queries;

mod exec;
use exec::*;

mod with_self;
pub use with_self::*;

use std::fmt::Display;

use super::{IntoSchemaManagerConnection, MigrationTrait, SchemaManager, seaql_migrations};
use crate::response::{AppliedData, LifecycleData, MigrationEntry, RolledBackData, StatusData};
use sea_orm::sea_query::IntoIden;
use sea_orm::{ConnectionTrait, DbErr, DynIden};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Status of migration
pub enum MigrationStatus {
    /// Not yet applied
    Pending,
    /// Applied
    Applied,
}

impl Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            MigrationStatus::Pending => "Pending",
            MigrationStatus::Applied => "Applied",
        };
        write!(f, "{status}")
    }
}

pub struct Migration {
    migration: Box<dyn MigrationTrait>,
    status: MigrationStatus,
}

impl Migration {
    /// Get migration name from MigrationName trait implementation
    pub fn name(&self) -> &str {
        self.migration.name()
    }

    /// Get migration status
    pub fn status(&self) -> MigrationStatus {
        self.status
    }
}

/// Performing migrations on a database
#[async_trait::async_trait]
pub trait MigratorTrait: Send {
    /// Vector of migrations in time sequence
    fn migrations() -> Vec<Box<dyn MigrationTrait>>;

    /// Name of the migration table, it is `seaql_migrations` by default
    fn migration_table_name() -> DynIden {
        seaql_migrations::Entity.into_iden()
    }

    /// Get list of migrations wrapped in `Migration` struct
    fn get_migration_files() -> Vec<Migration> {
        Self::migrations()
            .into_iter()
            .map(|migration| Migration {
                migration,
                status: MigrationStatus::Pending,
            })
            .collect()
    }

    /// Get list of applied migrations from database
    async fn get_migration_models<C>(db: &C) -> Result<Vec<seaql_migrations::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        get_migration_models(db, Self::migration_table_name()).await
    }

    /// Get list of migrations with status
    async fn get_migration_with_status<C>(db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        get_migration_with_status(
            Self::get_migration_files(),
            Self::get_migration_models(db).await?,
        )
    }

    /// Get list of pending migrations
    async fn get_pending_migrations<C>(db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        Ok(Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Pending)
            .collect())
    }

    /// Get list of applied migrations
    async fn get_applied_migrations<C>(db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        Ok(Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Applied)
            .collect())
    }

    /// Create migration table `seaql_migrations` in the database
    async fn install<C>(db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        install(db, Self::migration_table_name()).await
    }

    /// Check the status of all migrations
    async fn status<C>(db: &C) -> Result<StatusData, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        let migrations = Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .map(|m| MigrationEntry {
                name: m.migration.name().to_owned(),
                status: m.status.to_string(),
            })
            .collect();
        Ok(StatusData { migrations })
    }

    /// Drop all tables from the database, then reapply all migrations
    async fn fresh<'c, C>(db: C) -> Result<AppliedData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        exec_fresh::<Self>(&manager).await
    }

    /// Rollback all applied migrations, then reapply all migrations
    async fn refresh<'c, C>(db: C) -> Result<LifecycleData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down::<Self>(&manager, None).await?;
        let applied = exec_up::<Self>(&manager, None).await?;
        Ok::<_, DbErr>(LifecycleData {
            rolled_back,
            applied,
        })
    }

    /// Rollback all applied migrations
    async fn reset<'c, C>(db: C) -> Result<RolledBackData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down::<Self>(&manager, None).await?;
        uninstall(&manager, Self::migration_table_name()).await?;
        Ok::<_, DbErr>(RolledBackData { rolled_back })
    }

    /// Uninstall migration tracking table only (non-destructive)
    /// This will drop the `seaql_migrations` table but won't rollback other schema changes.
    async fn uninstall<'c, C>(db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        uninstall(&manager, Self::migration_table_name()).await
    }

    /// Apply pending migrations
    async fn up<'c, C>(db: C, steps: Option<u32>) -> Result<AppliedData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let applied = exec_up::<Self>(&manager, steps).await?;
        Ok(AppliedData { applied })
    }

    /// Rollback applied migrations
    async fn down<'c, C>(db: C, steps: Option<u32>) -> Result<RolledBackData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down::<Self>(&manager, steps).await?;
        Ok(RolledBackData { rolled_back })
    }
}

async fn exec_fresh<M>(manager: &SchemaManager<'_>) -> Result<AppliedData, DbErr>
where
    M: MigratorTrait + ?Sized,
{
    let db = manager.get_connection();
    M::install(db).await?;
    drop_everything(db).await?;
    let applied = exec_up::<M>(manager, None).await?;
    Ok(AppliedData { applied })
}

async fn exec_up<M>(manager: &SchemaManager<'_>, steps: Option<u32>) -> Result<Vec<String>, DbErr>
where
    M: MigratorTrait + ?Sized,
{
    let db = manager.get_connection();
    M::install(db).await?;
    exec_up_with(
        manager,
        steps,
        M::get_pending_migrations(db).await?,
        M::migration_table_name(),
    )
    .await
}

async fn exec_down<M>(manager: &SchemaManager<'_>, steps: Option<u32>) -> Result<Vec<String>, DbErr>
where
    M: MigratorTrait + ?Sized,
{
    let db = manager.get_connection();
    M::install(db).await?;
    exec_down_with(
        manager,
        steps,
        M::get_applied_migrations(db).await?,
        M::migration_table_name(),
    )
    .await
}
