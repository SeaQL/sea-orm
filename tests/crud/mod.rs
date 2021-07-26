use sea_orm::{entity::*, DbConn, InsertResult};

pub use super::common::bakery_chain::*;

pub mod create_baker;
pub mod create_cake;
pub mod create_lineitem;
pub mod create_order;
pub mod deletes;
pub mod updates;

pub async fn test_create_bakery(db: &DbConn) {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let res: InsertResult = Bakery::insert(seaside_bakery)
        .exec(db)
        .await
        .expect("could not insert bakery");

    let bakery: Option<bakery::Model> = Bakery::find_by_id(res.last_insert_id)
        .one(db)
        .await
        .expect("could not find bakery");

    assert!(bakery.is_some());
    let bakery_model = bakery.unwrap();
    assert_eq!(bakery_model.name, "SeaSide Bakery");
    assert_eq!(bakery_model.profit_margin, 10.4);
}

pub async fn test_create_customer(db: &DbConn) {
    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        notes: Set(Some("Loves cheese cake".to_owned())),
        ..Default::default()
    };
    let res: InsertResult = Customer::insert(customer_kate)
        .exec(db)
        .await
        .expect("could not insert customer");

    let customer: Option<customer::Model> = Customer::find_by_id(res.last_insert_id)
        .one(db)
        .await
        .expect("could not find customer");

    assert!(customer.is_some());
    let customer_model = customer.unwrap();
    assert_eq!(customer_model.name, "Kate");
    assert_eq!(customer_model.notes, Some("Loves cheese cake".to_owned()));
}
