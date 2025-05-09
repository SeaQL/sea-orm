use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Bakery::Table)
                    .col(pk_auto(Bakery::Id))
                    .col(string(Bakery::Name))
                    .col(double(Bakery::ProfitMargin))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bakery::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Bakery {
    Table,
    Id,
    Name,
    ProfitMargin,
}
