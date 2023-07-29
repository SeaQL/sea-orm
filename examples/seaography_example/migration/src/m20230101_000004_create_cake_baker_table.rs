use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CakeBaker::Table)
                    .col(ColumnDef::new(CakeBaker::CakeId).integer().not_null())
                    .col(ColumnDef::new(CakeBaker::BakerId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-cake_baker")
                            .col(CakeBaker::CakeId)
                            .col(CakeBaker::BakerId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake_baker-cake_id")
                            .from(CakeBaker::Table, CakeBaker::CakeId)
                            .to(Cake::Table, Cake::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake_baker-baker_id")
                            .from(CakeBaker::Table, CakeBaker::BakerId)
                            .to(Baker::Table, Baker::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CakeBaker::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CakeBaker {
    Table,
    CakeId,
    BakerId,
}

#[derive(DeriveIden)]
enum Baker {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Cake {
    Table,
    Id,
}
