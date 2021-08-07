pub use super::*;
use chrono::offset::Utc;
use rust_decimal_macros::dec;
use uuid::Uuid;

pub async fn test_create_lineitem(db: &DbConn) {
    // Bakery
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let bakery_insert_res: InsertResult = Bakery::insert(seaside_bakery)
        .exec(db)
        .await
        .expect("could not insert bakery");

    // Baker
    let baker_bob = baker::ActiveModel {
        name: Set("Baker Bob".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(bakery_insert_res.last_insert_id as i32)),
        ..Default::default()
    };
    let baker_insert_res: InsertResult = Baker::insert(baker_bob)
        .exec(db)
        .await
        .expect("could not insert baker");

    // Cake
    let mud_cake = cake::ActiveModel {
        name: Set("Mud Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(Some(bakery_insert_res.last_insert_id as i32)),
        ..Default::default()
    };

    let cake_insert_res: InsertResult = Cake::insert(mud_cake)
        .exec(db)
        .await
        .expect("could not insert cake");

    // Cake_Baker
    let cake_baker = cakes_bakers::ActiveModel {
        cake_id: Set(cake_insert_res.last_insert_id as i32),
        baker_id: Set(baker_insert_res.last_insert_id as i32),
        ..Default::default()
    };
    let _cake_baker_res: InsertResult = CakesBakers::insert(cake_baker)
        .exec(db)
        .await
        .expect("could not insert cake_baker");

    // Customer
    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        notes: Set(Some("Loves cheese cake".to_owned())),
        ..Default::default()
    };
    let customer_insert_res: InsertResult = Customer::insert(customer_kate)
        .exec(db)
        .await
        .expect("could not insert customer");

    // Order
    let order_1 = order::ActiveModel {
        bakery_id: Set(bakery_insert_res.last_insert_id as i32),
        customer_id: Set(customer_insert_res.last_insert_id as i32),
        total: Set(dec!(7.55)),
        placed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let order_insert_res: InsertResult = Order::insert(order_1)
        .exec(db)
        .await
        .expect("could not insert order");

    // Lineitem
    let lineitem_1 = lineitem::ActiveModel {
        cake_id: Set(cake_insert_res.last_insert_id as i32),
        order_id: Set(order_insert_res.last_insert_id as i32),
        price: Set(dec!(7.55)),
        quantity: Set(1),
        ..Default::default()
    };
    let lineitem_insert_res: InsertResult = Lineitem::insert(lineitem_1)
        .exec(db)
        .await
        .expect("could not insert lineitem");

    let lineitem: Option<lineitem::Model> =
        Lineitem::find_by_id(lineitem_insert_res.last_insert_id)
            .one(db)
            .await
            .expect("could not find lineitem");

    assert!(lineitem.is_some());
    let lineitem_model = lineitem.unwrap();

    assert_eq!(lineitem_model.price, dec!(7.55));

    let cake: Option<cake::Model> = Cake::find_by_id(lineitem_model.cake_id as u64)
        .one(db)
        .await
        .expect("could not find cake");

    let cake_model = cake.unwrap();
    assert_eq!(cake_model.name, "Mud Cake");

    let order: Option<order::Model> = Order::find_by_id(lineitem_model.order_id)
        .one(db)
        .await
        .expect("could not find order");

    let order_model = order.unwrap();
    assert_eq!(
        order_model.customer_id,
        customer_insert_res.last_insert_id as i32
    );
}
