use super::m20220118_000001_create_cake_table::Cake;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220118_000002_create_fruit_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Fruit::Table)
                    .col(
                        ColumnDef::new(Fruit::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Fruit::Name).string().not_null())
                    .col(ColumnDef::new(Fruit::CakeId).integer().not_null())
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

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Fruit {
    Table,
    Id,
    Name,
    CakeId,
}
