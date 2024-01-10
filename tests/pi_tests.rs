pub mod common;

use common::{features::*};
use pretty_assertions::assert_eq;
use rust_decimal_macros::dec;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};
use std::str::FromStr;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("pi_tests").await;
    create_tables(&ctx.db).await?;
    create_and_update_pi(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_and_update_pi(db: &DatabaseConnection) -> Result<(), DbErr> {
    let pi = pi::Model {
        id: 1,
        decimal: dec!(3.1415926536),
        big_decimal: BigDecimal::from_str("3.1415926536").unwrap(),
        decimal_opt: None,
        big_decimal_opt: None,
    };

    let res = pi.clone().into_active_model().insert(db).await?;

    let model = Pi::find().one(db).await?;
    assert_eq!(model, Some(res));
    assert_eq!(model, Some(pi.clone()));

    let res = pi::ActiveModel {
        decimal_opt: Set(Some(dec!(3.1415926536))),
        big_decimal_opt: Set(Some(BigDecimal::from_str("3.1415926536").unwrap())),
        ..pi.clone().into_active_model()
    }
    .update(db)
    .await?;

    let model = Pi::find().one(db).await?;
    assert_eq!(model, Some(res));
    assert_eq!(
        model,
        Some(pi::Model {
            id: 1,
            decimal: dec!(3.1415926536),
            big_decimal: BigDecimal::from_str("3.1415926536").unwrap(),
            decimal_opt: Some(dec!(3.1415926536)),
            big_decimal_opt: Some(BigDecimal::from_str("3.1415926536").unwrap()),
        })
    );

    Ok(())
}
