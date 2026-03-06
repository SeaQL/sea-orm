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
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("cake_name_index")
                    .table("cake")
                    .col("name")
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("cake").to_owned())
            .await?;

        if std::env::var_os("ABORT_MIGRATION").eq(&Some("YES".into())) {
            return Err(DbErr::Migration(
                "Abort migration and rollback changes".into(),
            ));
        }

        Ok(())
    }
}
