use sea_orm::{entity::*, DbConn, ExecErr, InsertResult};

pub use super::bakery_chain::*;

pub async fn create_bakery(db: &DbConn) -> Result<(), ExecErr> {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let _res: InsertResult = Bakery::insert(seaside_bakery).exec(db).await?;

    Ok(())
}
