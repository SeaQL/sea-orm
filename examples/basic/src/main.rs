//! Basic sea-orm example.

#![deny(missing_docs)]

use sea_orm::Database;

mod entity;
mod mutation;
mod query;

use entity::*;
use mutation::*;
use query::*;

#[tokio::main]
async fn main() {
    let db = Database::connect(
        std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://sea:sea@localhost/bakery".to_owned()),
    )
    .await
    .unwrap();

    println!("{db:?}\n");

    println!("===== =====\n");

    all_about_query(&db).await.unwrap();

    println!("===== =====\n");

    all_about_mutation(&db).await.unwrap();
}
