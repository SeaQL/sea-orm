#[cfg(feature = "cli")]
pub mod cli;
pub mod manager;
pub mod migrator;
pub mod prelude;
pub mod seaql_migrations;
pub mod util;

pub use manager::*;
pub use migrator::*;

pub use async_trait;
pub use sea_orm;
pub use sea_orm::sea_query;
pub use sea_orm::DbErr;

pub trait MigrationName {
    fn name(&self) -> &str;
}

/// The migration definition
#[async_trait::async_trait]
pub trait MigrationTrait: MigrationName + Send + Sync {
    /// Define actions to perform when applying the migration
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr>;

    /// Define actions to perform when rolling back the migration
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Migration("We Don't Do That Here".to_owned()))
    }
}
