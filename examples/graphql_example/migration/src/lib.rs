pub use sea_orm_migration::prelude::*;

mod m20220120_000001_create_note_table;
mod m20220120_000002_seed_notes;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220120_000001_create_note_table::Migration),
            Box::new(m20220120_000002_seed_notes::Migration),
        ]
    }
}
