#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{ConnectionTrait, DatabaseConnection, entity::prelude::*};

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("execute_unprepared_tests");
    create_tables(&ctx.db)?;
    execute_unprepared(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn execute_unprepared(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    db.execute_unprepared(
        [
            "INSERT INTO insert_default (id) VALUES (1), (2), (3), (4), (5)",
            "DELETE FROM insert_default WHERE id % 2 = 0",
        ]
        .join(";")
        .as_str(),
    )?;

    assert_eq!(
        Entity::find().all(db)?,
        [Model { id: 1 }, Model { id: 3 }, Model { id: 5 }]
    );

    Ok(())
}
