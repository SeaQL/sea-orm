pub use super::*;
use rust_decimal_macros::dec;
use uuid::Uuid;

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
    let uuid = Uuid::new_v4();

    let mud_cake = cake::ActiveModel {
        name: Set("Mud Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(uuid),
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
    assert_eq!(cake_model.price, dec!(10.25));
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
    assert_eq!(cake_model.serial, uuid);

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
