pub use sea_orm_migration::prelude::*;

mod {{datatime}}_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new({{datatime}}_create_table::Migration)]
    }
}
