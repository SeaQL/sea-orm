use super::m20220118_000001_create_cake_table::Cake;
use sea_orm_migration::sea_orm::DbBackend;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fruit::Table)
                    .col(pk_auto(Fruit::Id))
                    .col(string(Fruit::Name))
                    .col(integer(Fruit::CakeId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fruit-cake_id")
                            .from(Fruit::Table, Fruit::CakeId)
                            .to(Cake::Table, Cake::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() != DbBackend::Sqlite {
            manager
                .drop_foreign_key(
                    ForeignKey::drop()
                        .table(Fruit::Table)
                        .name("fk-fruit-cake_id")
                        .to_owned(),
                )
                .await?;
        }
        manager
            .drop_table(Table::drop().table(Fruit::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Fruit {
    Table,
    Id,
    Name,
    CakeId,
}
