pub use sea_orm_migration::prelude::*;

mod m20260816_154940_init_with_a_test;
mod m20260822_140231_rename_test1;
mod m20260822_142545_rename;
mod m20260822_164810_test_move;
mod m20260824_184319_mix;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260816_154940_init_with_a_test::Migration),
            Box::new(m20260822_140231_rename_test1::Migration),
            Box::new(m20260822_142545_rename::Migration),
            Box::new(m20260822_164810_test_move::Migration),
            Box::new(m20260824_184319_mix::Migration),
        ]
    }
}
