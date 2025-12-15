#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, entity::prelude::*, entity::*};

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("uuid_fmt_tests");
    create_tables(&ctx.db)?;
    insert_uuid_fmt(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn insert_uuid_fmt(db: &DatabaseConnection) -> Result<(), DbErr> {
    let uuid = Uuid::new_v4();

    let uuid_fmt = uuid_fmt::Model {
        id: 1,
        uuid,
        uuid_braced: uuid.braced(),
        uuid_hyphenated: uuid.hyphenated(),
        uuid_simple: uuid.simple(),
        uuid_urn: uuid.urn(),
    };

    let result = uuid_fmt.clone().into_active_model().insert(db)?;

    assert_eq!(result, uuid_fmt);

    Ok(())
}
