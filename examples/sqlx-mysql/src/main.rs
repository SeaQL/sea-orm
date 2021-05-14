use sea_orm::{ColumnTrait, Database, EntityTrait, QueryErr, SelectQuery};

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

    find_all(&db).await.unwrap();

    find_one(&db).await.unwrap();

    count_fruits_by_cake(&db).await.unwrap();
}

async fn find_all(db: &Database) -> Result<(), QueryErr> {
    print!("find all cakes: ");

    let cakes = cake::Entity::find().all(db).await?;

    println!();
    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }

    print!("find all fruits: ");

    let fruits = fruit::Entity::find().all(db).await?;

    println!();
    for ff in fruits.iter() {
        println!("{:?}", ff);
        println!();
    }

    Ok(())
}

async fn find_one(db: &Database) -> Result<(), QueryErr> {
    print!("find one by primary key: ");

    let cheese = cake::Entity::find_by(1).one(db).await?;

    println!();
    println!("{:?}", cheese);
    println!();

    print!("find one by like: ");

    let chocolate = cake::Entity::find()
        .filter(cake::Column::Name.contains("chocolate"))
        .one(db)
        .await?;

    println!();
    println!("{:?}", chocolate);
    println!();

    print!("find models belong to: ");

    let fruits = cheese.find_fruit().all(db).await?;

    println!();
    for ff in fruits.iter() {
        println!("{:?}", ff);
        println!();
    }

    Ok(())
}

async fn count_fruits_by_cake(db: &Database) -> Result<(), QueryErr> {
    #[derive(Debug)]
    struct SelectResult {
        name: String,
        num_of_fruits: i32,
    }

    {
        // TODO: implement with derive macro
        use sea_orm::{FromQueryResult, QueryResult, TypeErr};

        impl FromQueryResult for SelectResult {
            fn from_query_result(row: QueryResult, pre: &str) -> Result<Self, TypeErr> {
                Ok(Self {
                    name: row.try_get(pre, "name")?,
                    num_of_fruits: row.try_get(pre, "num_of_fruits")?,
                })
            }
        }
    }

    print!("count fruits by cake: ");

    let select = cake::Entity::find()
        .left_join(fruit::Entity)
        .select_only()
        .column(cake::Column::Name)
        .column_as(fruit::Column::Id.count(), "num_of_fruits")
        .group_by(cake::Column::Name);

    let results = select.into_model::<SelectResult>().all(db).await?;

    println!();
    for rr in results.iter() {
        println!("{:?}", rr);
        println!();
    }

    Ok(())
}
