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
                    .table("fruit")
                    .col(pk_auto("id"))
                    .col(string("name"))
                    .col(integer("cake_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fruit-cake_id")
                            .from("fruit", "cake_id")
                            .to("cake", "id"),
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
                        .table("fruit")
                        .name("fk-fruit-cake_id")
                        .to_owned(),
                )
                .await?;
        }
        manager
            .drop_table(Table::drop().table("fruit").to_owned())
            .await
    }
}
