use crate::common::migration::*;
use sea_orm_migration::{MigratorTraitSelf, prelude::*};

pub struct Migrator {
    pub use_transaction: Option<bool>,
    pub should_fail: bool,
}

#[async_trait::async_trait]
impl MigratorTraitSelf for Migrator {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250101_000001_create_test_table::Migration {
            use_transaction: self.use_transaction,
            should_fail: self.should_fail,
        })]
    }
}
