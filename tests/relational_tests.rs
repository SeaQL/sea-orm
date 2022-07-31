pub mod common;

pub use chrono::offset::Utc;
pub use common::{bakery_chain::*, setup::*, TestContext};
pub use rust_decimal::prelude::*;
pub use rust_decimal_macros::dec;
pub use sea_orm::{entity::*, query::*, DbErr, FromQueryResult};
pub use uuid::Uuid;

// Run the test locally:
// DATABASE_URL="mysql://root:@localhost" cargo test --features sqlx-mysql,runtime-async-std-native-tls --test relational_tests
#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn left_join() {
    let ctx = TestContext::new("test_left_join").await;
    create_tables(&ctx.db).await.unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let _baker_1 = baker::ActiveModel {
        name: Set("Baker 1".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(bakery.id)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    let _baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert baker");

    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        name: String,
        bakery_name: Option<String>,
    }

    let select = baker::Entity::find()
        .left_join(bakery::Entity)
        .select_only()
        .column(baker::Column::Name)
        .column_as(bakery::Column::Name, "bakery_name")
        .filter(baker::Column::Name.contains("Baker 1"));

    let result = select
        .into_model::<SelectResult>()
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.name.as_str(), "Baker 1");
    assert_eq!(result.bakery_name, Some("SeaSide Bakery".to_string()));

    let select = baker::Entity::find()
        .left_join(bakery::Entity)
        .select_only()
        .column(baker::Column::Name)
        .column_as(bakery::Column::Name, "bakery_name")
        .filter(baker::Column::Name.contains("Baker 2"));

    let result = select
        .into_model::<SelectResult>()
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.bakery_name, None);

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(feature = "sqlx-mysql", feature = "sqlx-postgres"))]
pub async fn right_join() {
    let ctx = TestContext::new("test_right_join").await;
    create_tables(&ctx.db).await.unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let _customer_jim = customer::ActiveModel {
        name: Set("Jim".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let _order = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(15.10)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    #[derive(FromQueryResult)]
    #[allow(dead_code)]
    struct SelectResult {
        name: String,
        order_total: Option<Decimal>,
    }

    let select = order::Entity::find()
        .right_join(customer::Entity)
        .select_only()
        .column(customer::Column::Name)
        .column_as(order::Column::Total, "order_total")
        .filter(customer::Column::Name.contains("Kate"));

    let result = select
        .into_model::<SelectResult>()
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.order_total, Some(dec!(15.10)));

    let select = order::Entity::find()
        .right_join(customer::Entity)
        .select_only()
        .column(customer::Column::Name)
        .column_as(order::Column::Total, "order_total")
        .filter(customer::Column::Name.contains("Jim"));

    let result = select
        .into_model::<SelectResult>()
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result.order_total, None);

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn inner_join() {
    let ctx = TestContext::new("test_inner_join").await;
    create_tables(&ctx.db).await.unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let _customer_jim = customer::ActiveModel {
        name: Set("Jim".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(15.10)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    let kate_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(100.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        name: String,
        order_total: Option<Decimal>,
    }

    let select = order::Entity::find()
        .inner_join(customer::Entity)
        .select_only()
        .column(customer::Column::Name)
        .column_as(order::Column::Total, "order_total");

    let results = select
        .into_model::<SelectResult>()
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(results.len(), 2);
    assert!((&results)
        .iter()
        .any(|result| result.name == customer_kate.name.clone()
            && result.order_total == Some(kate_order_1.total)));
    assert!((&results)
        .iter()
        .any(|result| result.name == customer_kate.name.clone()
            && result.order_total == Some(kate_order_2.total)));

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn group_by() {
    let ctx = TestContext::new("test_group_by").await;
    create_tables(&ctx.db).await.unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(99.95)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    let kate_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(200.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        name: String,
        number_orders: Option<i64>,
        total_spent: Option<Decimal>,
        min_spent: Option<Decimal>,
        max_spent: Option<Decimal>,
    }

    let select = customer::Entity::find()
        .left_join(order::Entity)
        .select_only()
        .column(customer::Column::Name)
        .column_as(order::Column::Total.count(), "number_orders")
        .column_as(order::Column::Total.sum(), "total_spent")
        .column_as(order::Column::Total.min(), "min_spent")
        .column_as(order::Column::Total.max(), "max_spent")
        .group_by(customer::Column::Name);

    let result = select
        .into_model::<SelectResult>()
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(result.name.as_str(), "Kate");
    assert_eq!(result.number_orders, Some(2));
    assert_eq!(
        result.total_spent,
        Some(kate_order_1.total + kate_order_2.total)
    );
    assert_eq!(
        result.min_spent,
        Some(kate_order_1.total.min(kate_order_2.total))
    );
    assert_eq!(
        result.max_spent,
        Some(kate_order_1.total.max(kate_order_2.total))
    );
    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn having() {
    // customers with orders with total equal to $90
    let ctx = TestContext::new("test_having").await;
    create_tables(&ctx.db).await.unwrap();

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(100.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    let _kate_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_kate.id),
        total: Set(dec!(12.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    let customer_bob = customer::ActiveModel {
        name: Set("Bob".to_owned()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert customer");

    let _bob_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_bob.id),
        total: Set(dec!(50.0)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    let _bob_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id),
        customer_id: Set(customer_bob.id),
        total: Set(dec!(50.0)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .expect("could not insert order");

    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        name: String,
        order_total: Option<Decimal>,
    }

    let results = customer::Entity::find()
        .inner_join(order::Entity)
        .select_only()
        .column(customer::Column::Name)
        .column_as(order::Column::Total, "order_total")
        .group_by(customer::Column::Name)
        .group_by(order::Column::Total)
        .having(order::Column::Total.gt(dec!(90.00)))
        .into_model::<SelectResult>()
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, customer_kate.name.clone());
    assert_eq!(results[0].order_total, Some(kate_order_1.total));

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn linked() -> Result<(), DbErr> {
    use common::bakery_chain::Order;
    use sea_orm::{SelectA, SelectB};
    use sea_query::{Alias, Expr};

    let ctx = TestContext::new("test_linked").await;
    create_tables(&ctx.db).await?;

    // SeaSide Bakery
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let seaside_bakery_res = Bakery::insert(seaside_bakery).exec(&ctx.db).await?;

    // Bob's Baker, Cake & Cake Baker
    let baker_bob = baker::ActiveModel {
        name: Set("Baker Bob".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(seaside_bakery_res.last_insert_id as i32)),
        ..Default::default()
    };
    let baker_bob_res = Baker::insert(baker_bob).exec(&ctx.db).await?;
    let mud_cake = cake::ActiveModel {
        name: Set("Mud Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(Some(seaside_bakery_res.last_insert_id as i32)),
        ..Default::default()
    };
    let mud_cake_res = Cake::insert(mud_cake).exec(&ctx.db).await?;
    let bob_cakes_bakers = cakes_bakers::ActiveModel {
        cake_id: Set(mud_cake_res.last_insert_id as i32),
        baker_id: Set(baker_bob_res.last_insert_id as i32),
    };
    CakesBakers::insert(bob_cakes_bakers).exec(&ctx.db).await?;

    // Bobby's Baker, Cake & Cake Baker
    let baker_bobby = baker::ActiveModel {
        name: Set("Baker Bobby".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+85212345678",
        })),
        bakery_id: Set(Some(seaside_bakery_res.last_insert_id as i32)),
        ..Default::default()
    };
    let baker_bobby_res = Baker::insert(baker_bobby).exec(&ctx.db).await?;
    let cheese_cake = cake::ActiveModel {
        name: Set("Cheese Cake".to_owned()),
        price: Set(dec!(20.5)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(Some(seaside_bakery_res.last_insert_id as i32)),
        ..Default::default()
    };
    let cheese_cake_res = Cake::insert(cheese_cake).exec(&ctx.db).await?;
    let bobby_cakes_bakers = cakes_bakers::ActiveModel {
        cake_id: Set(cheese_cake_res.last_insert_id as i32),
        baker_id: Set(baker_bobby_res.last_insert_id as i32),
    };
    CakesBakers::insert(bobby_cakes_bakers)
        .exec(&ctx.db)
        .await?;
    let chocolate_cake = cake::ActiveModel {
        name: Set("Chocolate Cake".to_owned()),
        price: Set(dec!(30.15)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(Some(seaside_bakery_res.last_insert_id as i32)),
        ..Default::default()
    };
    let chocolate_cake_res = Cake::insert(chocolate_cake).exec(&ctx.db).await?;
    let bobby_cakes_bakers = cakes_bakers::ActiveModel {
        cake_id: Set(chocolate_cake_res.last_insert_id as i32),
        baker_id: Set(baker_bobby_res.last_insert_id as i32),
    };
    CakesBakers::insert(bobby_cakes_bakers)
        .exec(&ctx.db)
        .await?;

    // Kate's Customer, Order & Line Item
    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        notes: Set(Some("Loves cheese cake".to_owned())),
        ..Default::default()
    };
    let customer_kate_res = Customer::insert(customer_kate).exec(&ctx.db).await?;
    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(seaside_bakery_res.last_insert_id as i32),
        customer_id: Set(customer_kate_res.last_insert_id as i32),
        total: Set(dec!(15.10)),
        placed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let kate_order_1_res = Order::insert(kate_order_1).exec(&ctx.db).await?;
    lineitem::ActiveModel {
        cake_id: Set(cheese_cake_res.last_insert_id as i32),
        order_id: Set(kate_order_1_res.last_insert_id as i32),
        price: Set(dec!(7.55)),
        quantity: Set(2),
        ..Default::default()
    }
    .save(&ctx.db)
    .await?;
    let kate_order_2 = order::ActiveModel {
        bakery_id: Set(seaside_bakery_res.last_insert_id as i32),
        customer_id: Set(customer_kate_res.last_insert_id as i32),
        total: Set(dec!(29.7)),
        placed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let kate_order_2_res = Order::insert(kate_order_2).exec(&ctx.db).await?;
    lineitem::ActiveModel {
        cake_id: Set(chocolate_cake_res.last_insert_id as i32),
        order_id: Set(kate_order_2_res.last_insert_id as i32),
        price: Set(dec!(9.9)),
        quantity: Set(3),
        ..Default::default()
    }
    .save(&ctx.db)
    .await?;

    // Kara's Customer, Order & Line Item
    let customer_kara = customer::ActiveModel {
        name: Set("Kara".to_owned()),
        notes: Set(Some("Loves all cakes".to_owned())),
        ..Default::default()
    };
    let customer_kara_res = Customer::insert(customer_kara).exec(&ctx.db).await?;
    let kara_order_1 = order::ActiveModel {
        bakery_id: Set(seaside_bakery_res.last_insert_id as i32),
        customer_id: Set(customer_kara_res.last_insert_id as i32),
        total: Set(dec!(15.10)),
        placed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let kara_order_1_res = Order::insert(kara_order_1).exec(&ctx.db).await?;
    lineitem::ActiveModel {
        cake_id: Set(mud_cake_res.last_insert_id as i32),
        order_id: Set(kara_order_1_res.last_insert_id as i32),
        price: Set(dec!(7.55)),
        quantity: Set(2),
        ..Default::default()
    }
    .save(&ctx.db)
    .await?;
    let kara_order_2 = order::ActiveModel {
        bakery_id: Set(seaside_bakery_res.last_insert_id as i32),
        customer_id: Set(customer_kara_res.last_insert_id as i32),
        total: Set(dec!(29.7)),
        placed_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    let kara_order_2_res = Order::insert(kara_order_2).exec(&ctx.db).await?;
    lineitem::ActiveModel {
        cake_id: Set(cheese_cake_res.last_insert_id as i32),
        order_id: Set(kara_order_2_res.last_insert_id as i32),
        price: Set(dec!(9.9)),
        quantity: Set(3),
        ..Default::default()
    }
    .save(&ctx.db)
    .await?;

    #[derive(Debug, FromQueryResult, PartialEq)]
    struct BakerLite {
        name: String,
    }

    #[derive(Debug, FromQueryResult, PartialEq)]
    struct CustomerLite {
        name: String,
    }

    let baked_for_customers: Vec<(BakerLite, Option<CustomerLite>)> = Baker::find()
        .find_also_linked(baker::BakedForCustomer)
        .select_only()
        .column_as(baker::Column::Name, (SelectA, baker::Column::Name))
        .column_as(
            Expr::tbl(Alias::new("r4"), customer::Column::Name).into_simple_expr(),
            (SelectB, customer::Column::Name),
        )
        .group_by(baker::Column::Id)
        .group_by(Expr::tbl(Alias::new("r4"), customer::Column::Id).into_simple_expr())
        .group_by(baker::Column::Name)
        .group_by(Expr::tbl(Alias::new("r4"), customer::Column::Name).into_simple_expr())
        .order_by_asc(baker::Column::Id)
        .order_by_asc(Expr::tbl(Alias::new("r4"), customer::Column::Id).into_simple_expr())
        .into_model()
        .all(&ctx.db)
        .await?;

    assert_eq!(
        baked_for_customers,
        vec![
            (
                BakerLite {
                    name: "Baker Bob".to_owned(),
                },
                Some(CustomerLite {
                    name: "Kara".to_owned(),
                })
            ),
            (
                BakerLite {
                    name: "Baker Bobby".to_owned(),
                },
                Some(CustomerLite {
                    name: "Kate".to_owned(),
                })
            ),
            (
                BakerLite {
                    name: "Baker Bobby".to_owned(),
                },
                Some(CustomerLite {
                    name: "Kara".to_owned(),
                })
            ),
        ]
    );

    let baker_bob = Baker::find()
        .filter(baker::Column::Id.eq(1))
        .one(&ctx.db)
        .await?
        .unwrap();

    let baker_bob_customers = baker_bob
        .find_linked(baker::BakedForCustomer)
        .all(&ctx.db)
        .await?;

    assert_eq!(
        baker_bob_customers,
        vec![customer::Model {
            id: 2,
            name: "Kara".to_owned(),
            notes: Some("Loves all cakes".to_owned()),
        }]
    );

    ctx.delete().await;

    Ok(())
}
