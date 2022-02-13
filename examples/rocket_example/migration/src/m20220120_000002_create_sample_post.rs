use entity::post::*;
use entity::sea_orm::{ActiveModelTrait, Set};
use sea_schema::migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220120_000002_create_sample_post"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        ActiveModel {
            title: Set("Testing".to_owned()),
            text: Set("SeaORM Example".to_owned()),
            ..Default::default()
        }
        .save(conn)
        .await
        .map(|_| ())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
