#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, bakery_chain::*, setup::*};
pub use sea_orm::entity::*;
pub use sea_orm::{ConnectionTrait, DbErr, QueryFilter};

#[sea_orm_macros::test]
pub fn stream() -> Result<(), DbErr> {
    let ctx = TestContext::new("stream");
    create_bakery_table(&ctx.db)?;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)?;

    let result = Bakery::find_by_id(bakery.id.clone().unwrap())
        .stream(&ctx.db)?
        .next()
        .unwrap()?;

    assert_eq!(result.id, bakery.id.unwrap());

    ctx.delete();

    Ok(())
}
