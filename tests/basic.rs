#[allow(unused_imports)]
use sea_orm::{entity::*, error::*, sea_query, tests_cfg::*, Database, DbConn};

// DATABASE_URL="sqlite::memory:" cargo test --features sqlx-sqlit,runtime-async-std --test basic
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-actix", actix_rt::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
#[cfg(feature = "sqlx-sqlite")]
async fn main() {
    use std::env;
    let base_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

    let db: DbConn = Database::connect(&base_url).await.unwrap();

    setup_schema(&db).await;

    crud_cake(&db).await.unwrap();
}

#[cfg(feature = "sqlx-sqlite")]
async fn setup_schema(db: &DbConn) {
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

    let builder = db.get_database_backend();
    let result = db.execute(builder.build(&stmt)).await;
    println!("Create table cake: {:?}", result);
}

#[cfg(feature = "sqlx-sqlite")]
async fn crud_cake(db: &DbConn) -> Result<(), DbErr> {
    let apple = cake::ActiveModel {
        name: Set("Apple Pie".to_owned()),
        ..Default::default()
    };

    let mut apple = apple.save(db).await?;

    println!();
    println!("Inserted: {:?}", apple);

    assert_eq!(
        cake::ActiveModel {
            id: Set(1),
            name: Set("Apple Pie".to_owned()),
        },
        apple
    );

    apple.name = Set("Lemon Tart".to_owned());

    let apple = apple.save(db).await?;

    println!();
    println!("Updated: {:?}", apple);

    let count = cake::Entity::find().count(db).await?;

    println!();
    println!("Count: {:?}", count);
    assert_eq!(count, 1);

    let apple = cake::Entity::find_by_id(1).one(db).await?;

    assert_eq!(
        Some(cake::Model {
            id: 1,
            name: "Lemon Tart".to_owned(),
        }),
        apple
    );

    let apple: cake::ActiveModel = apple.unwrap().into();

    let result = apple.delete(db).await?;

    println!();
    println!("Deleted: {:?}", result);

    let apple = cake::Entity::find_by_id(1).one(db).await?;

    assert_eq!(None, apple);

    let count = cake::Entity::find().count(db).await?;

    println!();
    println!("Count: {:?}", count);
    assert_eq!(count, 0);

    Ok(())
}
