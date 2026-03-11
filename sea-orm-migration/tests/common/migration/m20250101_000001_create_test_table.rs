use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;
use sea_orm_migration::sea_orm::DbBackend;

pub struct Migration {
    pub use_transaction: Option<bool>,
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
        let expect_txn = self
            .use_transaction
            .unwrap_or(manager.get_database_backend() == DbBackend::Postgres);
        assert_eq!(
            manager.get_connection().is_transaction(),
            expect_txn,
            "up: expected is_transaction() = {expect_txn}"
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
        let expect_txn = self
            .use_transaction
            .unwrap_or(manager.get_database_backend() == DbBackend::Postgres);
        assert_eq!(
            manager.get_connection().is_transaction(),
            expect_txn,
            "down: expected is_transaction() = {expect_txn}"
        );

        manager
            .drop_table(Table::drop().table("test_table").to_owned())
            .await
    }
}
