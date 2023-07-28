use crate::entity::{baker, bakery, cake};
use sea_orm::entity::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let seaside_bakery = bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        };
        bakery::Entity::insert(seaside_bakery).exec(db).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        cake::Entity::delete_many().exec(db).await?;
        baker::Entity::delete_many().exec(db).await?;
        bakery::Entity::delete_many().exec(db).await?;

        Ok(())
    }
}
