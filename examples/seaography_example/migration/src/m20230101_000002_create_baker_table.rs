use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Baker::Table)
                    .col(pk_auto(Baker::Id))
                    .col(string(Baker::Name))
                    .col(string(Baker::Contact))
                    .col(integer_null(Baker::BakeryId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-baker-bakery_id")
                            .from(Baker::Table, Baker::BakeryId)
                            .to(Bakery::Table, Bakery::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Baker::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Baker {
    Table,
    Id,
    Name,
    Contact,
    BakeryId,
}

#[derive(DeriveIden)]
enum Bakery {
    Table,
    Id,
}
