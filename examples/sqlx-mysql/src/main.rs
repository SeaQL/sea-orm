use sea_orm::{ColumnTrait, Database, EntityTrait, FromQueryResult, QueryErr, SelectHelper};

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

    if false {
        println!("===== =====\n");

        json_tests(&db).await.unwrap();
    }

    println!("===== =====\n");

    find_all_stream(&db).await.unwrap();

    println!("===== =====\n");

    find_first_page(&db).await.unwrap();

    println!("===== =====\n");

    find_num_page(&db).await.unwrap();
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

async fn json_tests(db: &Database) -> Result<(), QueryErr> {
    find_all_json(&db).await?;

    println!("===== =====\n");

    find_together_json(&db).await?;

    println!("===== =====\n");

    count_fruits_by_cake_json(&db).await?;

    Ok(())
}

async fn find_all_json(db: &Database) -> Result<(), QueryErr> {
    print!("find all cakes: ");

    let cakes = cake::Entity::find().into_json().all(db).await?;

    println!("\n{}\n", serde_json::to_string_pretty(&cakes).unwrap());

    print!("find all fruits: ");

    let fruits = fruit::Entity::find().into_json().all(db).await?;

    println!("\n{}\n", serde_json::to_string_pretty(&fruits).unwrap());

    Ok(())
}

async fn find_together_json(db: &Database) -> Result<(), QueryErr> {
    print!("find cakes and fruits: ");

    let cakes_fruits = cake::Entity::find()
        .left_join_and_select(fruit::Entity)
        .into_json()
        .all(db)
        .await?;

    println!(
        "\n{}\n",
        serde_json::to_string_pretty(&cakes_fruits).unwrap()
    );

    print!("find one cake and fruit: ");

    let cake_fruit = cake::Entity::find()
        .left_join_and_select(fruit::Entity)
        .into_json()
        .one(db)
        .await?;

    println!("\n{}\n", serde_json::to_string_pretty(&cake_fruit).unwrap());

    Ok(())
}

async fn count_fruits_by_cake_json(db: &Database) -> Result<(), QueryErr> {
    print!("count fruits by cake: ");

    let count = cake::Entity::find()
        .left_join(fruit::Entity)
        .select_only()
        .column(cake::Column::Name)
        .column_as(fruit::Column::Id.count(), "num_of_fruits")
        .group_by(cake::Column::Name)
        .into_json()
        .all(db)
        .await?;

    println!("\n{}\n", serde_json::to_string_pretty(&count).unwrap());

    Ok(())
}

async fn find_all_stream(db: &Database) -> Result<(), QueryErr> {
    use futures::TryStreamExt;
    use std::time::Duration;
    use async_std::task::sleep;

    println!("find all cakes: ");
    let mut cake_paginator = cake::Entity::find().paginate(db, 2);
    while let Some(cake_res) = cake_paginator.fetch_and_next().await? {
        for cake in cake_res {
            println!("{:?}", cake);
        }
    }

    println!();
    println!("find all fruits: ");
    let mut fruit_paginator = fruit::Entity::find().paginate(db, 2);
    while let Some(fruit_res) = fruit_paginator.fetch_and_next().await? {
        for fruit in fruit_res {
            println!("{:?}", fruit);
        }
    }

    println!();
    println!("find all fruits with stream: ");
    let mut fruit_stream = fruit::Entity::find().paginate(db, 2).into_stream();
    while let Some(fruits) = fruit_stream.try_next().await? {
        for fruit in fruits {
            println!("{:?}", fruit);
        }
        sleep(Duration::from_millis(250)).await;
    }

    println!();
    println!("find all fruits in json with stream: ");
    let mut json_stream = fruit::Entity::find().into_json().paginate(db, 2).into_stream();
    while let Some(jsons) = json_stream.try_next().await? {
        for json in jsons {
            println!("{:?}", json);
        }
        sleep(Duration::from_millis(250)).await;
    }

    Ok(())
}

async fn find_first_page(db: &Database) -> Result<(), QueryErr> {
    println!("fruits first page: ");
    let page = fruit::Entity::find().paginate(db, 2).fetch_page(0).await?;
    for fruit in page {
        println!("{:?}", fruit);
    }

    Ok(())
}

async fn find_num_page(db: &Database) -> Result<(), QueryErr> {
    println!("fruits number of page: ");
    let num_page = fruit::Entity::find().paginate(db, 2).count_page().await?;
    println!("{:?}", num_page);

    Ok(())
}
