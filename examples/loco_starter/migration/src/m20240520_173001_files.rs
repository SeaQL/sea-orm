use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto("files")
                    .col(pk_auto("id"))
                    .col(integer("notes_id"))
                    .col(string("file_path"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_files_notes_id")
                            .from("files", "notes_id")
                            .to("notes", "id"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("files").to_owned())
            .await
    }
}
