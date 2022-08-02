use sea_orm::tests_cfg::{cake, cake_filling};
use sea_orm::{Database, DbErr, EntityTrait, JoinType, QuerySelect, RelationTrait};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    tokio::spawn(async move {
        let _cakes = cake::Entity::find()
            .join_rev(JoinType::InnerJoin, cake_filling::Relation::Cake.def())
            .all(&db)
            .await
            .unwrap();
    })
    .await
    .unwrap();

    Ok(())
}
