use sea_orm::{tests_cfg::*, Database, Entity};

#[async_std::main]
async fn main() {
    let mut db = Database::default();
    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();
    println!("{:?}", db);
    println!("");

    let rows = cake::Cake::find().all(&db).await.unwrap();

    for row in rows.iter() {
        println!("{:?}", row);
        println!("");
    }
}
