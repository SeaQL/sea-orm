#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::entity::prelude::*;

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("insert_default_tests");
    create_tables(&ctx.db)?;
    create_insert_default(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn create_insert_default(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let active_model = ActiveModel {
        ..Default::default()
    };

    active_model.clone().insert(db)?;
    active_model.clone().insert(db)?;
    active_model.insert(db)?;

    assert_eq!(
        Entity::find().all(db)?,
        [Model { id: 1 }, Model { id: 2 }, Model { id: 3 }]
    );

    Ok(())
}
