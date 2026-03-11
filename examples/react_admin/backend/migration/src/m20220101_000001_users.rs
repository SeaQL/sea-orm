use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto("users")
            .col(pk_auto("id"))
            .col(uuid("pid"))
            .col(string_uniq("email"))
            .col(string("password"))
            .col(string("api_key").unique_key())
            .col(string("name"))
            .col(string_null("reset_token"))
            .col(timestamp_null("reset_sent_at"))
            .col(string_null("email_verification_token"))
            .col(timestamp_null("email_verification_sent_at"))
            .col(timestamp_null("email_verified_at"))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("users").to_owned())
            .await
    }
}
