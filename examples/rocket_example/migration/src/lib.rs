pub use sea_schema::migration::prelude::*;

mod m20220120_000001_create_post_table;
mod m20220120_000002_create_sample_post;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220120_000001_create_post_table::Migration),
            Box::new(m20220120_000002_create_sample_post::Migration),
        ]
    }
}
