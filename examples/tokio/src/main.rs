mod cake;
use sea_orm::*;

#[tokio::main]
pub async fn main() {
    let db = Database::connect("sql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    tokio::spawn(async move {
        cake::Entity::find().one(&db).await.unwrap();
    })
    .await.unwrap();
}