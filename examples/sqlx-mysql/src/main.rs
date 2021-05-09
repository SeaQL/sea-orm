use sea_orm::{Database, EntityTrait};

mod example_cake;
mod example_fruit;

use example_cake as cake;
use example_fruit as fruit;

#[async_std::main]
async fn main() {
    let mut db = Database::default();
    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();
    println!("{:?}", db);
    println!();

    println!("find all");

    let cakes = cake::Entity::find().all(&db).await.unwrap();

    println!();
    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }

    println!("find one by primary key");

    let cheese = cake::Entity::find_one(&db, 1).await.unwrap();

    println!();
    println!("{:?}", cheese);
    println!();

    println!("find models belongs to");

    let fruits = cheese.find_fruit().all(&db).await.unwrap();

    println!();
    for ff in fruits.iter() {
        println!("{:?}", ff);
        println!();
    }
}
