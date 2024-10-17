mod entity;
use entity::prelude::*;
use sea_orm::{Database, DbBackend, EntityTrait, QueryTrait};

#[tokio::main]
async fn main() {
    let db = Database::connect(
        "postgres://postgres:password@127.0.0.1:5432/sea-orm-test?sslmode=disable",
    )
    .await
    .unwrap();
    println!("{}", VehicleVerify::find().build(DbBackend::Postgres));
    VehicleVerify::find().all(&db).await.unwrap();
}
