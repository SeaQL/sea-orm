mod cake;
use sea_orm::*;

#[tokio::main]
pub async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let db = Database::connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    tokio::spawn(async move {
        cake::Entity::find().one(&db).await.unwrap();
    })
    .await.unwrap();
}