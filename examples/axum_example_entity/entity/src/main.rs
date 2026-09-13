use entity::Entities;

#[tokio::main]
async fn main() {
    sea_orm_migration::entity_cli::run_cli(Entities, migration::Migrator).await;
}
