use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220118_000001_create_cake_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Cake::Table)
                    .col(
                        ColumnDef::new(Cake::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Cake::Name).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cake::Table).to_owned())
            .await
    }
}

/// `Iden` is a trait for identifiers used in any query statement.
/// 
/// Commonly implemented by Enum where each Enum represents a table found in a database,
/// and its variants include table name and column name.
/// 
/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Cake {
    Table,
    Id,
    Name,
}
