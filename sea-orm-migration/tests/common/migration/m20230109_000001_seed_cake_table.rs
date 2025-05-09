use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{entity::*, query::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let transaction = db.begin().await?;

        cake::ActiveModel {
            name: Set("Cheesecake".to_owned()),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;

        if std::env::var_os("ABORT_MIGRATION").eq(&Some("YES".into())) {
            return Err(DbErr::Migration(
                "Abort migration and rollback changes".into(),
            ));
        }

        transaction.commit().await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let transaction = db.begin().await?;

        cake::Entity::delete_many()
            .filter(cake::Column::Name.eq("Cheesecake"))
            .exec(&transaction)
            .await?;

        transaction.commit().await?;

        Ok(())
    }
}

mod cake {
    use sea_orm_migration::sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
