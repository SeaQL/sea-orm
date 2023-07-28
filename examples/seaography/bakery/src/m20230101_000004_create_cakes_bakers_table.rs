use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CakesBakers::Table)
                    .col(ColumnDef::new(CakesBakers::CakeId).integer().not_null())
                    .col(ColumnDef::new(CakesBakers::BakerId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-cakes_bakers")
                            .col(CakesBakers::CakeId)
                            .col(CakesBakers::BakerId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cakes_bakers-cake_id")
                            .from(CakesBakers::Table, CakesBakers::CakeId)
                            .to(Cake::Table, Cake::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cakes_bakers-baker_id")
                            .from(CakesBakers::Table, CakesBakers::BakerId)
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
            .drop_table(Table::drop().table(CakesBakers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CakesBakers {
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
