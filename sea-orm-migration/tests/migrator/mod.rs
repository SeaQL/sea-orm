use sea_orm_migration::prelude::*;

mod m20220118_000001_create_cake_table;
mod m20220118_000002_create_fruit_table;
mod m20220118_000003_seed_cake_table;
mod m20220923_000001_seed_cake_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220118_000001_create_cake_table::Migration),
            Box::new(m20220118_000002_create_fruit_table::Migration),
            Box::new(m20220118_000003_seed_cake_table::Migration),
            Box::new(m20220923_000001_seed_cake_table::Migration),
        ]
    }
}
