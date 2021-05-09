use sea_orm::{Database, EntityTrait};

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

    print!("find all cakes: ");

    let cakes = cake::Entity::find().all(&db).await.unwrap();

    println!();
    for cc in cakes.iter() {
        println!("{:?}", cc);
        println!();
    }

    print!("find all fruits: ");

    let fruits = fruit::Entity::find().all(&db).await.unwrap();

    println!();
    for cc in fruits.iter() {
        println!("{:?}", cc);
        println!();
    }

    print!("find one by primary key: ");

    let cheese = cake::Entity::find_by(1).one(&db).await.unwrap();

    println!();
    println!("{:?}", cheese);
    println!();

    print!("find models belong to: ");

    let fruits = cheese.find_fruit().all(&db).await.unwrap();

    println!();
    for ff in fruits.iter() {
        println!("{:?}", ff);
        println!();
    }

    count_fruits_by_cake(&db).await;
}

async fn count_fruits_by_cake(db: &Database) {
    #[derive(Debug)]
    struct SelectResult {
        name: String,
        num_of_fruits: i32,
    }

    {
        // TODO: implement with derive macro
        use sea_orm::{FromQueryResult, QueryResult, TypeErr};

        impl FromQueryResult for SelectResult {
            fn from_query_result(row: QueryResult) -> Result<Self, TypeErr> {
                Ok(Self {
                    name: row.try_get("name")?,
                    num_of_fruits: row.try_get("num_of_fruits")?,
                })
            }
        }
    }

    print!("count fruits by cake: ");

    let mut select = cake::Entity::find().left_join(cake::Relation::Fruit);
    {
        use sea_orm::sea_query::*;
        type Cake = cake::Column;
        type Fruit = fruit::Column;

        select
            .query()
            .clear_selects()
            .column((cake::Entity, Cake::Name))
            .expr_as(
                Expr::tbl(fruit::Entity, Fruit::Id).count(),
                Alias::new("num_of_fruits"),
            )
            .group_by_col((cake::Entity, Cake::Name));
    }

    let results = select.into_model::<SelectResult>().all(db).await.unwrap();

    println!();
    for rr in results.iter() {
        println!("{:?}", rr);
        println!();
    }
}
