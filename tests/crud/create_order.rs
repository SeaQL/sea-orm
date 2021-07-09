pub use super::*;
use rust_decimal_macros::dec;

pub async fn test_create_order(db: &DbConn) {
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
        notes: Set("Loves cheese cake".to_owned()),
        ..Default::default()
    };
    let customer_insert_res: InsertResult = Customer::insert(customer_kate)
        .exec(db)
        .await
        .expect("could not insert customer");

    // Order
    let order_1 = order::ActiveModel {
        bakery_id: Set(Some(bakery_insert_res.last_insert_id as i32)),
        customer_id: Set(Some(customer_insert_res.last_insert_id as i32)),
        total: Set(dec!(15.10)),
        placed_at: Set("placeholder".to_string()),
        ..Default::default()
    };
    let order_insert_res: InsertResult = Order::insert(order_1)
        .exec(db)
        .await
        .expect("could not insert order");

    // Lineitem
    let lineitem_1 = lineitem::ActiveModel {
        cake_id: Set(Some(cake_insert_res.last_insert_id as i32)),
        order_id: Set(Some(order_insert_res.last_insert_id as i32)),
        price: Set(dec!(7.55)),
        quantity: Set(2),
        ..Default::default()
    };
    let _lineitem_insert_res: InsertResult = Lineitem::insert(lineitem_1)
        .exec(db)
        .await
        .expect("could not insert lineitem");

    let order: Option<order::Model> = Order::find_by_id(order_insert_res.last_insert_id)
        .one(db)
        .await
        .expect("could not find order");

    assert!(order.is_some());
    let order_model = order.unwrap();
    assert_eq!(order_model.total, dec!(15.10));

    let customer: Option<customer::Model> = Customer::find_by_id(order_model.customer_id)
        .one(db)
        .await
        .expect("could not find customer");

    let customer_model = customer.unwrap();
    assert_eq!(customer_model.name, "Kate");

    let bakery: Option<bakery::Model> = Bakery::find_by_id(order_model.bakery_id)
        .one(db)
        .await
        .expect("could not find bakery");

    let bakery_model = bakery.unwrap();
    assert_eq!(bakery_model.name, "SeaSide Bakery");

    let related_lineitems: Vec<lineitem::Model> = order_model
        .find_related(Lineitem)
        .all(db)
        .await
        .expect("could not find related lineitems");
    assert_eq!(related_lineitems.len(), 1);
    assert_eq!(related_lineitems[0].price, dec!(7.55));
    assert_eq!(related_lineitems[0].quantity, 2);
}
