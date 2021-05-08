use sea_orm::{tests_cfg::*, Database, EntityTrait};

#[async_std::main]
async fn main() {
    let mut db = Database::default();
    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();
    println!("{:?}", db);
    println!();

    let cakes = cake::Entity::find().all(&db).await.unwrap();

    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }
}
