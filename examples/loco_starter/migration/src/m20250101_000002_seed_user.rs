use super::m20220101_000001_users::Users;
use sea_orm_migration::prelude::*;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::insert()
            .into_table(Users::Table)
            .columns([
                Users::Pid,
                Users::Email,
                Users::Password,
                Users::ApiKey,
                Users::Name,
            ])
            .values_panic([
                Uuid::new_v4().into(),
                "demo@sea-ql.org".into(),
                "$argon2id$v=19$m=19456,t=2,p=1$VgMk6uAFpORoiUoxIRT3Ww$+Z8K1Ef6wn0/n6UXWc5Wf3Gn18nhUcc7HFfGjX3hnWU".into(),
                format!("lo-{}", Uuid::new_v4()).into(),
                "Demo User".into(),
            ])
            .to_owned();
        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
