use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cake::Table)
                    .col(pk_auto(Cake::Id))
                    .col(string(Cake::Name))
                    .col(decimal_len(Cake::Price, 16, 4))
                    .col(integer(Cake::BakeryId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake-bakery_id")
                            .from(Cake::Table, Cake::BakeryId)
                            .to(Bakery::Table, Bakery::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(boolean(Cake::GlutenFree))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cake::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Cake {
    Table,
    Id,
    Name,
    Price,
    GlutenFree,
    BakeryId,
}

#[derive(DeriveIden)]
enum Bakery {
    Table,
    Id,
}
