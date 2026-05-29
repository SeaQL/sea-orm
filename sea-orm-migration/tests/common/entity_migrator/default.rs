use sea_orm_migration::prelude::*;

use crate::common::entity_migration::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250101_000001_create_cake_table::Migration),
            Box::new(m20250101_000002_create_fruit_table::Migration),
        ]
    }
}
