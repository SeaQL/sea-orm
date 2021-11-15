mod material;
use sea_orm::*;

#[async_std::main]
pub async fn main() {
    let db = Database::connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    async_std::task::spawn(async move {
        material::Entity::find().one(&db).await.unwrap();
    })
    .await;
}
