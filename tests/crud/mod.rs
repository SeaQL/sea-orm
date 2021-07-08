use sea_orm::{entity::*, DbConn, InsertResult};

pub use super::bakery_chain::*;

pub mod create_lineitem;
pub mod create_order;

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
        notes: Set("Loves cheese cake".to_owned()),
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
    assert_eq!(customer_model.notes, "Loves cheese cake");
}

pub async fn test_create_cake(db: &DbConn) {
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
    let baker_insert_res: InsertResult = Baker::insert(baker_bob)
        .exec(db)
        .await
        .expect("could not insert baker");

    let mud_cake = cake::ActiveModel {
        name: Set("Mud Cake".to_owned()),
        price: Set(10.25),
        gluten_free: Set(false),
        bakery_id: Set(Some(bakery_insert_res.last_insert_id as i32)),
        ..Default::default()
    };

    let cake_insert_res: InsertResult = Cake::insert(mud_cake)
        .exec(db)
        .await
        .expect("could not insert cake");

    let cake: Option<cake::Model> = Cake::find_by_id(cake_insert_res.last_insert_id)
        .one(db)
        .await
        .expect("could not find cake");

    let cake_baker = cakes_bakers::ActiveModel {
        cake_id: Set(cake_insert_res.last_insert_id as i32),
        baker_id: Set(baker_insert_res.last_insert_id as i32),
        ..Default::default()
    };
    let _cake_baker_res: InsertResult = CakesBakers::insert(cake_baker)
        .exec(db)
        .await
        .expect("could not insert cake_baker");

    assert!(cake.is_some());
    let cake_model = cake.unwrap();
    assert_eq!(cake_model.name, "Mud Cake");
    assert_eq!(cake_model.price, 10.25);
    assert_eq!(cake_model.gluten_free, false);
    assert_eq!(
        cake_model
            .find_related(Bakery)
            .one(db)
            .await
            .expect("Bakery not found")
            .unwrap()
            .name,
        "SeaSide Bakery"
    );

    let related_bakers: Vec<baker::Model> = cake_model
        .find_related(Baker)
        .all(db)
        .await
        .expect("could not find related bakers");
    assert_eq!(related_bakers.len(), 1);
    assert_eq!(related_bakers[0].name, "Baker Bob");

    let baker: Option<baker::Model> = Baker::find_by_id(baker_insert_res.last_insert_id)
        .one(db)
        .await
        .expect("could not find baker");

    let related_cakes: Vec<cake::Model> = baker
        .unwrap()
        .find_related(Cake)
        .all(db)
        .await
        .expect("could not find related cakes");
    assert_eq!(related_cakes.len(), 1);
    assert_eq!(related_cakes[0].name, "Mud Cake")
}
