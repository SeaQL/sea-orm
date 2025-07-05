pub use sea_orm_migration::prelude::*;

mod m20250629_081733_post_seeder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> =
            vec![Box::new(m20250629_081733_post_seeder::Migration)];

        migrations.extend(migration::Migrator::migrations());
        migrations
    }
}
