use crate::*;
use sea_orm::{entity::*, query::*, Database};

pub async fn all_about_operation(db: &Database) -> Result<(), ExecErr> {
    save_active_model(db).await?;

    save_custom_active_model(db).await?;

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
    println!("Updated: {:?}\n", pineapple);

    Ok(())
}
