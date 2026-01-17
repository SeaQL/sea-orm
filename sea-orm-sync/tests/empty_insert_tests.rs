#![allow(unused_imports, dead_code)]

pub mod common;
mod crud;

pub use common::{TestContext, bakery_chain::*, setup::*};
pub use sea_orm::{
    DatabaseConnection, DbBackend, EntityName, ExecResult, entity::*, error::DbErr, tests_cfg,
};

pub use crud::*;
// use common::bakery_chain::*;
use sea_orm::{DbConn, TryInsertResult};

#[sea_orm_macros::test]
fn main() {
    let ctx = TestContext::new("bakery_chain_empty_insert_tests");
    create_tables(&ctx.db).unwrap();
    test(&ctx.db);
    ctx.delete();
}

pub fn test(db: &DbConn) {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };

    let res = Bakery::insert(seaside_bakery).exec(db);

    assert!(res.is_ok());

    let double_seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        id: Set(1),
        ..Default::default()
    };

    let conflict_insert = Bakery::insert_many([double_seaside_bakery])
        .on_conflict_do_nothing()
        .exec(db);

    assert!(matches!(conflict_insert, Ok(TryInsertResult::Conflicted)));

    let empty_insert = Bakery::insert_many(std::iter::empty::<bakery::ActiveModel>())
        .exec(db)
        .unwrap();

    assert!(empty_insert.last_insert_id.is_none());
}
