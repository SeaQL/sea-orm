mod cake;
use sea_orm::*;

#[tokio::main]
pub async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .init();

    let db = Database::connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    tokio::spawn(async move {
        cake::Entity::find().one(&db).await.unwrap();
    })
    .await.unwrap();
}