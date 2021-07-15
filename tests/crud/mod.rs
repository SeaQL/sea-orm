use sea_orm::{entity::*, DbConn, InsertResult};

pub use super::common::bakery_chain::*;

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

pub async fn test_create_baker(db: &DbConn) {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let bakery_insert_res: InsertResult = Bakery::insert(seaside_bakery)
        .exec(db)
        .await
        .expect("could not insert bakery");

    let baker_bob = baker::ActiveModel {
        name: Set("Baker Bob".to_owned()),
        bakery_id: Set(Some(bakery_insert_res.last_insert_id as i32)),
        ..Default::default()
    };
    let res: InsertResult = Baker::insert(baker_bob)
        .exec(db)
        .await
        .expect("could not insert baker");

    let baker: Option<baker::Model> = Baker::find_by_id(res.last_insert_id)
        .one(db)
        .await
        .expect("could not find baker");

    assert!(baker.is_some());
    let baker_model = baker.unwrap();
    assert_eq!(baker_model.name, "Baker Bob");
    assert_eq!(
        baker_model
            .find_related(Bakery)
            .one(db)
            .await
            .expect("Bakery not found")
            .unwrap()
            .name,
        "SeaSide Bakery"
    );

    let bakery: Option<bakery::Model> = Bakery::find_by_id(bakery_insert_res.last_insert_id)
        .one(db)
        .await
        .unwrap();

    let related_bakers: Vec<baker::Model> = bakery
        .unwrap()
        .find_related(Baker)
        .all(db)
        .await
        .expect("could not find related bakers");
    assert_eq!(related_bakers.len(), 1);
    assert_eq!(related_bakers[0].name, "Baker Bob")
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
