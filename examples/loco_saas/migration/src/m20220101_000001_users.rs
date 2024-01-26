use std::borrow::BorrowMut;

use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = table_auto(Users::Table)
            .col(pk_auto(Users::Id).borrow_mut())
            .col(uuid(Users::Pid).borrow_mut())
            .col(string_uniq(Users::Email).borrow_mut())
            .col(string(Users::Password).borrow_mut())
            .col(string(Users::ApiKey).borrow_mut().unique_key())
            .col(string(Users::Name).borrow_mut())
            .col(string_null(Users::ResetToken).borrow_mut())
            .col(timestamp_null(Users::ResetSentAt).borrow_mut())
            .col(string_null(Users::EmailVerificationToken).borrow_mut())
            .col(timestamp_null(Users::EmailVerificationSentAt).borrow_mut())
            .col(timestamp_null(Users::EmailVerifiedAt).borrow_mut())
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
