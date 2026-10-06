use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

pub struct Migration {
    pub use_transaction: Option<bool>,
    pub expect_transaction: bool,
    pub should_fail: bool,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250101_000001_create_test_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    fn use_transaction(&self) -> Option<bool> {
        self.use_transaction
    }

    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        assert_eq!(
            manager.get_connection().is_transaction(),
            self.expect_transaction,
            "up: unexpected transaction state"
        );

        manager
            .create_table(
                Table::create()
                    .table("test_table")
                    .col(pk_auto("id"))
                    .col(string("name"))
                    .to_owned(),
            )
            .await?;

        if self.should_fail {
            return Err(DbErr::Migration("intentional failure".into()));
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        assert_eq!(
            manager.get_connection().is_transaction(),
            self.expect_transaction,
            "down: unexpected transaction state"
        );

        manager
            .drop_table(Table::drop().table("test_table").to_owned())
            .await
    }
}
