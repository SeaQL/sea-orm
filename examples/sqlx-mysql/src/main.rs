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

    print!("find all cakes: ");

    let cakes = cake::Entity::find().all(&db).await.unwrap();

    println!();
    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }

    print!("find all fruits: ");

    let fruits = fruit::Entity::find().all(&db).await.unwrap();

    println!();
    for cc in fruits.iter() {
        println!("{:?}", cc);
        println!();
    }

    print!("find one by primary key: ");

    let cheese = cake::Entity::find_by(1).one(&db).await.unwrap();

    println!();
    println!("{:?}", cheese);
    println!();

    print!("find models belong to: ");

    let fruits = cheese.find_fruit().all(&db).await.unwrap();

    println!();
    for ff in fruits.iter() {
        println!("{:?}", ff);
        println!();
    }
}
