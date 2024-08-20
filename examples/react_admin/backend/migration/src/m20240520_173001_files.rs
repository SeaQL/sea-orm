use sea_orm_migration::{prelude::*, schema::*};

use super::m20231103_114510_notes::Notes;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Files::Table)
                    .col(pk_auto(Files::Id))
                    .col(integer(Files::NotesId))
                    .col(string(Files::FilePath))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_files_notes_id")
                            .from(Files::Table, Files::NotesId)
                            .to(Notes::Table, Notes::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Files {
    Table,
    Id,
    NotesId,
    FilePath,
}
