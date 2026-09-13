#[cfg(feature = "cli")]
pub mod cli;
pub mod connection;
pub mod manager;
pub mod migrator;
pub mod prelude;
pub mod response;
pub mod schema;
pub mod seaql_migrations;
pub mod util;

#[cfg(feature = "entity-first")]
pub mod codegen;
#[cfg(feature = "entity-first")]
pub mod entity_cli;
#[cfg(feature = "entity-first")]
pub mod fs;
#[cfg(feature = "entity-first")]
pub mod summary;

pub use connection::*;
pub use manager::*;
pub use migrator::*;

pub use async_trait;
pub use sea_orm;
pub use sea_orm::DbErr;
pub use sea_orm::sea_query;

#[cfg(feature = "entity-first")]
pub use sea_orm::schema::SchemaBuilder;

/// Trait for a set of entities to be registered into a [`SchemaBuilder`].
///
/// Implement this on a unit struct in your entity crate:
///
/// ```rust,ignore
/// pub struct Entities;
///
/// impl sea_orm_migration::EntitySet for Entities {
///     fn register(self, builder: sea_orm_migration::SchemaBuilder) -> sea_orm_migration::SchemaBuilder {
///         builder
///             .register(user::Entity)
///             .register(post::Entity)
///     }
/// }
/// ```
#[cfg(feature = "entity-first")]
pub trait EntitySet {
    fn register(self, builder: SchemaBuilder) -> SchemaBuilder;
}

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

    /// Control whether this migration runs inside a transaction.
    ///
    /// - `None` (default): follow backend convention (Postgres = transaction, MySQL/SQLite = no transaction)
    /// - `Some(true)`: force wrapping in a transaction on any backend
    /// - `Some(false)`: disable automatic transaction wrapping (use `manager.begin()` for manual control)
    fn use_transaction(&self) -> Option<bool> {
        None
    }
}
