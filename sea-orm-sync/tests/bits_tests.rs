#![allow(unused_imports, dead_code)]

pub mod common;

use common::features::*;
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, entity::prelude::*, entity::*};

#[sea_orm_macros::test]
#[cfg(feature = "sqlx-postgres")]
fn main() -> Result<(), DbErr> {
    let ctx = common::TestContext::new("bits_tests");
    create_bits_table(&ctx.db)?;
    create_and_update(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn create_and_update(db: &DatabaseConnection) -> Result<(), DbErr> {
    let bits = bits::Model {
        id: 1,
        bit0: 0,
        bit1: 1,
        bit8: 8,
        bit16: 16,
        bit32: 32,
        bit64: 64,
    };

    let res = bits.clone().into_active_model().insert(db)?;

    let model = Bits::find().one(db)?;
    assert_eq!(model, Some(res));
    assert_eq!(model, Some(bits.clone()));

    let res = bits::ActiveModel {
        bit32: Set(320),
        bit64: Set(640),
        ..bits.clone().into_active_model()
    }
    .update(db)?;

    let model = Bits::find().one(db)?;
    assert_eq!(model, Some(res));
    assert_eq!(
        model,
        Some(bits::Model {
            id: 1,
            bit0: 0,
            bit1: 1,
            bit8: 8,
            bit16: 16,
            bit32: 320,
            bit64: 640,
        })
    );

    Ok(())
}
