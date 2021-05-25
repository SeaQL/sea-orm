use sea_orm::{ColumnTrait, Database, EntityTrait, FromQueryResult, QueryErr, QueryHelper};

mod example_cake;
mod example_cake_filling;
mod example_filling;
mod example_fruit;

use example_cake as cake;
use example_cake_filling as cake_filling;
use example_filling as filling;
use example_fruit as fruit;

#[async_std::main]
async fn main() {
    let mut db = Database::default();

    db.connect("mysql://sea:sea@localhost/bakery")
        .await
        .unwrap();

    println!("{:?}\n", db);

    println!("===== =====\n");

    find_all(&db).await.unwrap();

    println!("===== =====\n");

    find_together(&db).await.unwrap();

    println!("===== =====\n");

    find_one(&db).await.unwrap();

    println!("===== =====\n");

    count_fruits_by_cake(&db).await.unwrap();

    println!("===== =====\n");

    find_many_to_many(&db).await.unwrap();

    println!("===== =====\n");

    find_all_json(&db).await.unwrap();

    println!("===== =====\n");

    find_one_json(&db).await.unwrap();
}

async fn find_all(db: &Database) -> Result<(), QueryErr> {
    print!("find all cakes: ");

    let cakes = cake::Entity::find().all(db).await?;

    println!();
    for cc in cakes.iter() {
        println!("{:?}\n", cc);
    }

    print!("find all fruits: ");

    let fruits = fruit::Entity::find().all(db).await?;

    println!();
    for ff in fruits.iter() {
        println!("{:?}\n", ff);
    }

    Ok(())
}

async fn find_together(db: &Database) -> Result<(), QueryErr> {
    print!("find cakes and fruits: ");

    let both = cake::Entity::find()
        .left_join_and_select(fruit::Entity)
        .all(db)
        .await?;

    println!();
    for bb in both.iter() {
        println!("{:?}\n", bb);
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
        println!("{:?}\n", ff);
    }

    Ok(())
}

async fn count_fruits_by_cake(db: &Database) -> Result<(), QueryErr> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        name: String,
        num_of_fruits: i32,
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
        println!("{:?}\n", rr);
    }

    Ok(())
}

async fn find_many_to_many(db: &Database) -> Result<(), QueryErr> {
    print!("find cakes and fillings: ");

    let both = cake::Entity::find()
        .left_join_and_select(filling::Entity)
        .all(db)
        .await?;

    println!();
    for bb in both.iter() {
        println!("{:?}\n", bb);
    }

    print!("find fillings for cheese cake: ");

    let cheese = cake::Entity::find_by(1).one(db).await?;

    let fillings: Vec<filling::Model> = cheese.find_filling().all(db).await?;

    println!();
    for ff in fillings.iter() {
        println!("{:?}\n", ff);
    }

    print!("find cakes for lemon: ");

    let lemon = filling::Entity::find_by(2).one(db).await?;

    let cakes: Vec<cake::Model> = lemon.find_cake().all(db).await?;

    println!();
    for cc in cakes.iter() {
        println!("{:?}\n", cc);
    }

    Ok(())
}

async fn find_all_json(db: &Database) -> Result<(), QueryErr> {
    print!("find all cakes: ");

    let cakes = cake::Entity::find().as_json().all(db).await?;

    println!("\n{:?}\n", cakes);

    print!("find all fruits: ");

    let fruits = fruit::Entity::find().as_json().all(db).await?;

    println!("\n{:?}\n", fruits);

    Ok(())
}

async fn find_one_json(db: &Database) -> Result<(), QueryErr> {
    print!("find one by primary key: ");

    let cheese = cake::Entity::find_by(1).as_json().one(db).await?;

    println!("\n{:?}\n", cheese);

    print!("find one by like: ");

    let chocolate = cake::Entity::find()
        .filter(cake::Column::Name.contains("chocolate"))
        .as_json()
        .one(db)
        .await?;

    println!("\n{:?}\n", chocolate);

    Ok(())
}
