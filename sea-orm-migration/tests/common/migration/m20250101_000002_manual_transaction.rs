use sea_orm_migration::prelude::*;
use sea_orm_migration::schema::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250101_000002_manual_transaction"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    fn use_transaction(&self) -> Option<bool> {
        Some(false)
    }

    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        assert!(
            !manager.get_connection().is_transaction(),
            "outer manager should not be in a transaction"
        );

        let m = manager.begin().await?;
        assert!(
            m.get_connection().is_transaction(),
            "inner manager should be in a transaction"
        );
        m.create_table(
            Table::create()
                .table("manual_txn_table")
                .col(pk_auto("id"))
                .col(string("name"))
                .to_owned(),
        )
        .await?;
        m.commit().await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let m = manager.begin().await?;
        m.drop_table(Table::drop().table("manual_txn_table").to_owned())
            .await?;
        m.commit().await?;

        Ok(())
    }
}
