#![allow(unused_imports, dead_code)]

pub mod common;

pub use sea_orm::{Database, DbConn, entity::*, error::*, query::*, sea_query, tests_cfg::*};

// DATABASE_URL=sqlite::memory: cargo test --features rusqlite --test basic
// DATABASE_URL=sqlite::memory: cargo test --features sqlx-sqlite,runtime-tokio --test basic
#[sea_orm_macros::test]
#[cfg(feature = "rusqlite")]
fn main() -> Result<(), DbErr> {
    dotenv::from_filename(".env.local").ok();
    dotenv::from_filename(".env").ok();

    let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

    let db: DbConn = Database::connect(&base_url)?;
    setup_schema(&db)?;
    crud_cake(&db)?;

    Ok(())
}

#[cfg(feature = "rusqlite")]
fn setup_schema(db: &DbConn) -> Result<(), DbErr> {
    use sea_query::*;

    let stmt = sea_query::Table::create()
        .table(cake::Entity)
        .col(
            ColumnDef::new(cake::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(cake::Column::Name).string())
        .to_owned();

    let result = db.execute(&stmt)?;
    println!("Create table cake: {result:?}");

    Ok(())
}

#[cfg(feature = "rusqlite")]
fn crud_cake(db: &DbConn) -> Result<(), DbErr> {
    let apple = cake::ActiveModel {
        name: Set("Apple Pie".to_owned()),
        ..Default::default()
    };

    let mut apple = apple.save(db)?;

    println!();
    println!("Inserted: {apple:?}");

    assert_eq!(
        apple,
        cake::ActiveModel {
            id: Unchanged(1),
            name: Unchanged("Apple Pie".to_owned()),
        }
    );

    apple.name = Set("Lemon Tart".to_owned());

    let apple = apple.save(db)?;

    println!();
    println!("Updated: {apple:?}");

    let count = cake::Entity::find().count(db)?;

    println!();
    println!("Count: {count:?}");
    assert_eq!(count, 1);

    let apple = cake::Entity::find_by_id(1).one(db)?;

    assert_eq!(
        Some(cake::Model {
            id: 1,
            name: "Lemon Tart".to_owned(),
        }),
        apple
    );

    let apple: cake::Model = apple.unwrap();

    let result = apple.delete(db)?;

    println!();
    println!("Deleted: {result:?}");

    let apple = cake::Entity::find_by_id(1).one(db)?;

    assert_eq!(None, apple);

    let count = cake::Entity::find().count(db)?;

    println!();
    println!("Count: {count:?}");
    assert_eq!(count, 0);

    Ok(())
}
