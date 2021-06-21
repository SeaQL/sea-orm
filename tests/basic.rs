use sea_orm::{entity::*, query::*, sea_query, tests_cfg::*, DbConn};

mod setup;

#[async_std::test]
// cargo test --test basic -- --nocapture
async fn main() {
    let db: DbConn = setup::setup().await;

    setup_schema(&db).await;

    crud_cake(&db).await.unwrap();
}

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
        .build(SqliteQueryBuilder);

    let result = db.execute(stmt.into()).await;
    println!("Create table cake: {:?}", result);
}

async fn crud_cake(db: &DbConn) -> Result<(), ExecErr> {
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

    let apple = cake::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(|_| ExecErr)?;

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

    let apple = cake::Entity::find_by_id(1)
        .one(db)
        .await
        .map_err(|_| ExecErr)?;

    assert_eq!(None, apple);

    Ok(())
}
