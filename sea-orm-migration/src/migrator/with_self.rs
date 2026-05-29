use super::{Migration, MigrationStatus, exec::*};
use crate::{
    IntoSchemaManagerConnection, MigrationTrait, SchemaManager,
    response::{AppliedData, LifecycleData, MigrationEntry, RolledBackData, StatusData, fnv64_hex},
    seaql_migrations,
};
use sea_orm::sea_query::IntoIden;
use sea_orm::{ConnectionTrait, DbErr, DynIden};

/// Performing migrations on a database
#[async_trait::async_trait]
pub trait MigratorTraitSelf: Sized + Send + Sync {
    /// Vector of migrations in time sequence
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>>;

    /// Name of the migration table, it is `seaql_migrations` by default
    fn migration_table_name(&self) -> DynIden {
        seaql_migrations::Entity.into_iden()
    }

    /// FNV64 hex digest of the ordered list of migration names.
    /// Used as the `migrations_hash` in JSON responses so callers can detect
    /// binary/config drift.
    fn migrations_hash(&self) -> String {
        let names: Vec<String> = self
            .migrations()
            .iter()
            .map(|m| m.name().to_owned())
            .collect();
        fnv64_hex(names.iter().map(String::as_str))
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
    async fn status<C>(&self, db: &C) -> Result<StatusData, DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;
        let migrations = self
            .get_migration_with_status(db)
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
    async fn fresh<'c, C>(&self, db: C) -> Result<AppliedData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        exec_fresh(self, &manager).await
    }

    /// Rollback all applied migrations, then reapply all migrations
    async fn refresh<'c, C>(&self, db: C) -> Result<LifecycleData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down(self, &manager, None).await?;
        let applied = exec_up(self, &manager, None).await?;
        Ok::<_, DbErr>(LifecycleData {
            rolled_back,
            applied,
        })
    }

    /// Rollback all applied migrations
    async fn reset<'c, C>(&self, db: C) -> Result<RolledBackData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down(self, &manager, None).await?;
        uninstall(&manager, self.migration_table_name()).await?;
        Ok::<_, DbErr>(RolledBackData { rolled_back })
    }

    /// Uninstall migration tracking table only (non-destructive)
    async fn uninstall<'c, C>(&self, db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        uninstall(&manager, self.migration_table_name()).await
    }

    /// Apply pending migrations
    async fn up<'c, C>(&self, db: C, steps: Option<u32>) -> Result<AppliedData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let applied = exec_up(self, &manager, steps).await?;
        Ok(AppliedData { applied })
    }

    /// Rollback applied migrations
    async fn down<'c, C>(&self, db: C, steps: Option<u32>) -> Result<RolledBackData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down(self, &manager, steps).await?;
        Ok(RolledBackData { rolled_back })
    }
}

#[async_trait::async_trait]
impl<M> MigratorTraitSelf for M
where
    M: super::MigratorTrait + Sized + Send + Sync,
{
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        M::migrations()
    }

    fn migration_table_name(&self) -> DynIden {
        M::migration_table_name()
    }

    fn get_migration_files(&self) -> Vec<Migration> {
        M::get_migration_files()
    }

    async fn get_migration_models<C>(&self, db: &C) -> Result<Vec<seaql_migrations::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        M::get_migration_models(db).await
    }

    async fn get_migration_with_status<C>(&self, db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        M::get_migration_with_status(db).await
    }

    async fn get_pending_migrations<C>(&self, db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        M::get_pending_migrations(db).await
    }

    async fn get_applied_migrations<C>(&self, db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        M::get_applied_migrations(db).await
    }

    async fn install<C>(&self, db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        M::install(db).await
    }

    async fn status<C>(&self, db: &C) -> Result<StatusData, DbErr>
    where
        C: ConnectionTrait,
    {
        self.install(db).await?;
        let migrations = self
            .get_migration_with_status(db)
            .await?
            .into_iter()
            .map(|m| MigrationEntry {
                name: m.migration.name().to_owned(),
                status: m.status.to_string(),
            })
            .collect();
        Ok(StatusData { migrations })
    }

    async fn fresh<'c, C>(&self, db: C) -> Result<AppliedData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let applied = exec_fresh(self, &manager).await?;
        Ok::<_, DbErr>(applied)
    }

    async fn refresh<'c, C>(&self, db: C) -> Result<LifecycleData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down(self, &manager, None).await?;
        let applied = exec_up(self, &manager, None).await?;
        Ok::<_, DbErr>(LifecycleData {
            rolled_back,
            applied,
        })
    }

    async fn reset<'c, C>(&self, db: C) -> Result<RolledBackData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down(self, &manager, None).await?;
        uninstall(&manager, self.migration_table_name()).await?;
        Ok::<_, DbErr>(RolledBackData { rolled_back })
    }

    async fn uninstall<'c, C>(&self, db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        M::uninstall(db).await
    }

    async fn up<'c, C>(&self, db: C, steps: Option<u32>) -> Result<AppliedData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let applied = exec_up(self, &manager, steps).await?;
        Ok(AppliedData { applied })
    }

    async fn down<'c, C>(&self, db: C, steps: Option<u32>) -> Result<RolledBackData, DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_database_executor();
        let manager = SchemaManager::new(db);
        let rolled_back = exec_down(self, &manager, steps).await?;
        Ok(RolledBackData { rolled_back })
    }
}

async fn exec_fresh<M>(migrator: &M, manager: &SchemaManager<'_>) -> Result<AppliedData, DbErr>
where
    M: MigratorTraitSelf,
{
    let db = manager.get_connection();
    migrator.install(db).await?;
    drop_everything(db).await?;
    let applied = exec_up(migrator, manager, None).await?;
    Ok(AppliedData { applied })
}

async fn exec_up<M>(
    migrator: &M,
    manager: &SchemaManager<'_>,
    steps: Option<u32>,
) -> Result<Vec<String>, DbErr>
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
) -> Result<Vec<String>, DbErr>
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
