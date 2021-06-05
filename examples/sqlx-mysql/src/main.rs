use sea_orm::Database;

mod example_cake;
mod example_cake_filling;
mod example_filling;
mod example_fruit;
mod select;
mod operation;

use example_cake as cake;
use example_cake_filling as cake_filling;
use example_filling as filling;
use example_fruit as fruit;
use select::*;
use operation::*;

#[async_std::main]
async fn main() {
    let mut db = Database::default();

    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    println!("{:?}\n", db);

    println!("===== =====\n");

    all_about_select(&db).await.unwrap();

    println!("===== =====\n");

    all_about_operation(&db).await.unwrap();
}
