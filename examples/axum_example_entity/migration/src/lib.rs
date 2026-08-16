pub use sea_orm_migration::prelude::*;

mod m20260816_154940_init_with_a_test;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260816_154940_init_with_a_test::Migration),
        ]
    }
}
