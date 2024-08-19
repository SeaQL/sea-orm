use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto(Users::Table)
            .col(pk_auto(Users::Id))
            .col(uuid(Users::Pid))
            .col(string_uniq(Users::Email))
            .col(string(Users::Password))
            .col(string(Users::ApiKey).unique_key())
            .col(string(Users::Name))
            .col(string_null(Users::ResetToken))
            .col(timestamp_null(Users::ResetSentAt))
            .col(string_null(Users::EmailVerificationToken))
            .col(timestamp_null(Users::EmailVerificationSentAt))
            .col(timestamp_null(Users::EmailVerifiedAt))
            .to_owned();
        manager.create_table(table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    Pid,
    Email,
    Name,
    Password,
    ApiKey,
    ResetToken,
    ResetSentAt,
    EmailVerificationToken,
    EmailVerificationSentAt,
    EmailVerifiedAt,
}
