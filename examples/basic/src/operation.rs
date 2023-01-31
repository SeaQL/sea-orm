use super::*;
use sea_orm::{entity::*, error::*, DbConn};

pub async fn all_about_operation(db: &DbConn) -> Result<(), DbErr> {
    insert_and_update(db).await?;

    println!("===== =====\n");

    save_active_model(db).await?;

    println!("===== =====\n");

    save_custom_active_model(db).await?;

    Ok(())
}

pub async fn insert_and_update(db: &DbConn) -> Result<(), DbErr> {
    let pear = fruit::ActiveModel {
        name: Set("pear".to_owned()),
        ..Default::default()
    };
    let res = Fruit::insert(pear).exec(db).await?;

    println!();
    println!("Inserted: last_insert_id = {}\n", res.last_insert_id);

    let pear: Option<fruit::Model> = Fruit::find_by_id(res.last_insert_id).one(db).await?;

    println!();
    println!("Pear: {pear:?}\n");

    let mut pear: fruit::ActiveModel = pear.unwrap().into();
    pear.name = Set("Sweet pear".to_owned());

    let pear: fruit::Model = pear.update(db).await?;

    println!();
    println!("Updated: {pear:?}\n");

    Ok(())
}

pub async fn save_active_model(db: &DbConn) -> Result<(), DbErr> {
    let banana = fruit::ActiveModel {
        name: Set("Banana".to_owned()),
        ..Default::default()
    };
    let mut banana: fruit::ActiveModel = banana.save(db).await?;

    println!();
    println!("Inserted: {banana:?}\n");

    banana.name = Set("Banana Mongo".to_owned());

    let banana: fruit::ActiveModel = banana.save(db).await?;

    println!();
    println!("Updated: {banana:?}\n");

    let result = banana.delete(db).await?;

    println!();
    println!("Deleted: {result:?}\n");

    Ok(())
}

mod form {
    use super::fruit::*;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveIntoActiveModel)]
    pub struct InputModel {
        pub name: String,
    }
}

async fn save_custom_active_model(db: &DbConn) -> Result<(), DbErr> {
    let pineapple = form::InputModel {
        name: "Pineapple".to_owned(),
    }
    .into_active_model();

    let pineapple = pineapple.save(db).await?;

    println!();
    println!("Saved: {pineapple:?}\n");

    Ok(())
}
