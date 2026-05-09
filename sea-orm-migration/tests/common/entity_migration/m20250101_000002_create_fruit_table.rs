use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("fruit"))
                    .if_not_exists()
                    .col(pk_auto(Alias::new("id")))
                    .col(string(Alias::new("name")))
                    .col(integer(Alias::new("cake_id")))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-fruit-cake_id")
                            .from(Alias::new("fruit"), Alias::new("cake_id"))
                            .to(Alias::new("cake"), Alias::new("id")),
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
                        .table(Alias::new("fruit"))
                        .name("fk-fruit-cake_id")
                        .to_owned(),
                )
                .await?;
        }
        manager
            .drop_table(Table::drop().table(Alias::new("fruit")).to_owned())
            .await
    }
}
