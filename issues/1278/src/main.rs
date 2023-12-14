use sea_orm::{
    Database, DeriveColumn, EntityTrait, EnumIter, FromQueryResult, QuerySelect,
};

mod entity;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    PoolName,
}

#[derive(Debug, FromQueryResult)]
struct PoolResult {
    name: String,
}

#[tokio::main]
async fn main() {
    let db = Database::connect("xxxx").await.unwrap();

    let result1 = entity::Entity::find()
        .select_only()
        .column(entity::Column::Name)
        .into_model::<PoolResult>()
        .all(&db)
        .await
        .unwrap();

    let result2: Vec<String> = entity::Entity::find()
        .select_only()
        .column_as(entity::Column::Name, QueryAs::PoolName)
        .into_values::<_, QueryAs>()
        .all(&db)
        .await
        .unwrap();
}