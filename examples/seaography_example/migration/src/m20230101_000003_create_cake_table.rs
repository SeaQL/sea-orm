use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("cake")
                    .col(pk_auto("id"))
                    .col(string("name"))
                    .col(decimal_len("price", 16, 4))
                    .col(integer("bakery_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-cake-bakery_id")
                            .from("cake", "bakery_id")
                            .to("bakery", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(boolean("gluten_free"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("cake").to_owned())
            .await
    }
}
