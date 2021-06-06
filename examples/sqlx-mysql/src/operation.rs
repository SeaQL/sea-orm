use super::*;
use sea_orm::{entity::*, query::*, Database};

pub async fn all_about_operation(db: &Database) -> Result<(), ExecErr> {
    insert_and_update(db).await?;

    println!("===== =====\n");

    save_active_model(db).await?;

    println!("===== =====\n");

    save_custom_active_model(db).await?;

    Ok(())
}

pub async fn insert_and_update(db: &Database) -> Result<(), ExecErr> {
    let pear = fruit::ActiveModel {
        name: Val::set("pear".to_owned()),
        ..Default::default()
    };
    let res = Fruit::insert(pear).exec(db).await?;

    println!();
    println!("Inserted: {:?}\n", res);

    let pear = Fruit::find_by(res.last_insert_id)
        .one(db)
        .await
        .map_err(|_| ExecErr)?;

    println!();
    println!("Pear: {:?}\n", pear);

    let mut pear: fruit::ActiveModel = pear.unwrap().into();
    pear.name = Val::set("Sweet pear".to_owned());

    let res = Fruit::update(pear).exec(db).await?;

    println!();
    println!("Updated: {:?}\n", res);

    Ok(())
}

pub async fn save_active_model(db: &Database) -> Result<(), ExecErr> {
    let banana = fruit::ActiveModel {
        name: Val::set("banana".to_owned()),
        ..Default::default()
    };
    let mut banana = banana.save(db).await?;

    println!();
    println!("Inserted: {:?}\n", banana);

    banana.name = Val::set("banana banana".to_owned());

    let banana = banana.save(db).await?;

    println!();
    println!("Updated: {:?}\n", banana);

    Ok(())
}

mod form {
    use super::fruit::*;
    use sea_orm::entity::prelude::*;

    #[derive(
        Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, DeriveActiveModelBehavior,
    )]
    pub struct Model {
        pub id: i32,
        pub name: String,
    }
}

async fn save_custom_active_model(db: &Database) -> Result<(), ExecErr> {
    let pineapple = form::ActiveModel {
        id: Val::unset(),
        name: Val::set("pineapple".to_owned()),
    };

    let pineapple = pineapple.save(db).await?;

    println!();
    println!("Saved: {:?}\n", pineapple);

    Ok(())
}
