//! Basic sea-orm example.

#![deny(missing_docs)]

use sea_orm::Database;

mod entities;
pub mod example_cake;
pub mod example_cake_filling;
pub mod example_filling;
pub mod example_fruit;
mod operation;
pub mod sea_orm_active_enums;
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
    let db = Database::connect("mysql://sea:sea@localhost/test")
        .await
        .unwrap();

    println!("{db:?}\n");

    println!("===== =====\n");

    all_about_select(&db).await.unwrap();
}
