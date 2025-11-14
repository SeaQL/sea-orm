use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let insert = Query::insert()
            .into_table("cake")
            .columns(["name"])
            .values_panic(["Tiramisu".into()])
            .to_owned();

        manager.execute(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let delete = Query::delete()
            .from_table("cake")
            .and_where(Expr::col("name").eq("Tiramisu"))
            .to_owned();

        manager.execute(delete).await?;

        Ok(())
    }
}
