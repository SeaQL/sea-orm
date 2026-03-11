use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("cake_baker")
                    .col(integer("cake_id"))
                    .col(integer("baker_id"))
                    .primary_key(
                        Index::create()
                            .name("pk-cake_baker")
                            .col("cake_id")
                            .col("baker_id"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake_baker-cake_id")
                            .from("cake_baker", "cake_id")
                            .to("cake", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake_baker-baker_id")
                            .from("cake_baker", "baker_id")
                            .to("baker", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("cake_baker").to_owned())
            .await
    }
}
