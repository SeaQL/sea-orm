pub use sea_orm_migration::prelude::*;

mod entity;
mod m20230101_000001_create_bakery_table;
mod m20230101_000002_create_baker_table;
mod m20230101_000003_create_cake_table;
mod m20230101_000004_create_cakes_bakers_table;
mod m20230101_000005_create_customer_table;
mod m20230101_000006_create_order_table;
mod m20230101_000007_create_lineitem_table;
mod m20230102_000001_seed_bakery_data;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230101_000001_create_bakery_table::Migration),
            Box::new(m20230101_000002_create_baker_table::Migration),
            Box::new(m20230101_000003_create_cake_table::Migration),
            Box::new(m20230101_000004_create_cakes_bakers_table::Migration),
            // Box::new(m20230101_000005_create_customer_table::Migration),
            // Box::new(m20230101_000006_create_order_table::Migration),
            // Box::new(m20230101_000007_create_lineitem_table::Migration),
            Box::new(m20230102_000001_seed_bakery_data::Migration),
        ]
    }
}
