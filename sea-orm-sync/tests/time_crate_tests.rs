#![allow(unused_imports, dead_code)]

pub mod common;
pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{DatabaseConnection, IntoActiveModel, entity::prelude::*};
use serde_json::json;
use time::macros::{date, time};

#[sea_orm_macros::test]
fn main() {
    let ctx = TestContext::new("time_crate_tests");
    create_tables(&ctx.db).unwrap();
    create_transaction_log(&ctx.db).unwrap();

    ctx.delete();
}

pub fn create_transaction_log(db: &DatabaseConnection) -> Result<(), DbErr> {
    let transaction_log = transaction_log::Model {
        id: 1,
        date: date!(2022 - 03 - 13),
        time: time!(16:24:00),
        date_time: date!(2022 - 03 - 13).with_time(time!(16:24:00)),
        date_time_tz: date!(2022 - 03 - 13)
            .with_time(time!(16:24:00))
            .assume_utc(),
    };

    let res = TransactionLog::insert(transaction_log.clone().into_active_model()).exec(db)?;

    assert_eq!(transaction_log.id, res.last_insert_id);
    assert_eq!(
        TransactionLog::find().one(db)?,
        Some(transaction_log.clone())
    );

    let json = TransactionLog::find().into_json().one(db)?.unwrap();

    #[cfg(feature = "sqlx-postgres")]
    assert_eq!(
        json,
        json!({
            "id": 1,
            "date": "2022-03-13",
            "time": "16:24:00",
            "date_time": "2022-03-13T16:24:00",
            "date_time_tz": "2022-03-13T16:24:00Z",
        })
    );

    #[cfg(feature = "sqlx-mysql")]
    assert_eq!(
        json,
        json!({
            "id": 1,
            "date": "2022-03-13",
            "time": "16:24:00",
            "date_time": "2022-03-13T16:24:00",
            "date_time_tz": "2022-03-13T16:24:00Z",
        })
    );

    #[cfg(all(not(feature = "sync"), feature = "sqlx-sqlite"))]
    assert_eq!(
        json,
        json!({
            "id": 1,
            "date": "2022-03-13",
            "time": "16:24:00.0",
            "date_time": "2022-03-13 16:24:00.0",
            "date_time_tz": "2022-03-13T16:24:00Z",
        })
    );

    #[cfg(feature = "rusqlite")]
    assert_eq!(
        json,
        json!({
            "id": 1,
            "date": "2022-03-13",
            "time": "16:24:00.0",
            "date_time": "2022-03-13 16:24:00.0",
            "date_time_tz": "2022-03-13 16:24:00.0+00:00",
        })
    );

    assert_ne!(json, "");

    Ok(())
}
