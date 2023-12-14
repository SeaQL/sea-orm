use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cake::Table)
                    .col(
                        ColumnDef::new(Cake::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Cake::Name).string().not_null())
                    .col(ColumnDef::new(Cake::Price).decimal_len(19, 4).not_null())
                    .col(ColumnDef::new(Cake::BakeryId).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake-bakery_id")
                            .from(Cake::Table, Cake::BakeryId)
                            .to(Bakery::Table, Bakery::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Cake::GlutenFree).boolean().not_null())
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
