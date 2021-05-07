use sea_orm::{tests_cfg::*, Database, Entity};

#[async_std::main]
async fn main() {
    let mut db = Database::default();
    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();
    println!("{:?}", db);
    println!();

    let cakes = cake::Cake::find().all(&db).await.unwrap();

    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }
}
