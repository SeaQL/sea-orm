use sea_orm::{entity::*, error::*, DbConn, InsertResult};

pub use super::bakery_chain::*;

pub async fn create_bakery(db: &DbConn) -> Result<(), DbErr> {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let res: InsertResult = Bakery::insert(seaside_bakery).exec(db).await?;

    let bakery: Option<bakery::Model> = Bakery::find_by_id(res.last_insert_id).one(db).await?;

    assert!(bakery.is_some());
    let bakery_model = bakery.unwrap();
    assert_eq!(bakery_model.name, "SeaSide Bakery");
    assert_eq!(bakery_model.profit_margin, 10.4);

    Ok(())
}
