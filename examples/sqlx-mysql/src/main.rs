use sea_orm::Database;

mod entities;
mod example_cake;
mod example_cake_filling;
mod example_filling;
mod example_fruit;
mod operation;
mod select;

use entities::*;
use example_cake as cake;
use example_cake_filling as cake_filling;
use example_filling as filling;
use example_fruit as fruit;
use operation::*;
use select::*;

#[async_std::main]
async fn main() {
    let db = Database::connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    println!("{:?}\n", db);

    println!("===== =====\n");

    all_about_select(&db).await.unwrap();

    println!("===== =====\n");

    all_about_operation(&db).await.unwrap();
}
