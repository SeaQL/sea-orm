use sea_orm::{tests_cfg::*, Database, EntityTrait};

#[async_std::main]
async fn main() {
    let mut db = Database::default();
    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();
    println!("{:?}", db);
    println!();

    println!("find all");
    println!();

    let cakes = cake::Entity::find().all(&db).await.unwrap();

    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }

    println!("find one by primary key");
    println!();

    let cheese = cake::Entity::find_one(&db, 1).await.unwrap();

    println!("{:?}", cheese);
    println!();
}
