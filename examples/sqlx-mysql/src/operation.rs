use crate::*;
use sea_orm::{entity::*, query::*, Database};

pub async fn all_about_operation(db: &Database) -> Result<(), ExecErr> {
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