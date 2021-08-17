use chrono::offset::Utc;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use sea_orm::{entity::*, query::*, FromQueryResult};

pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};

// Run the test locally:
// DATABASE_URL="mysql://root:@localhost" cargo test --features sqlx-mysql,runtime-async-std --test relational_tests
#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn left_join() {
    let ctx = TestContext::new("test_left_join").await;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let _baker_1 = baker::ActiveModel {
        name: Set("Baker 1".to_owned()),
        contact_details: Set(serde_json::json!({
            "mobile": "+61424000000",
            "home": "0395555555",
            "address": "12 Test St, Testville, Vic, Australia"
        })),
        bakery_id: Set(Some(bakery.id.clone().unwrap())),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert baker");

    let _baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
        contact_details: Set(serde_json::json!({})),
        bakery_id: Set(None),
        ..Default::default()
    }
    .save(&ctx.db)
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

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let _customer_jim = customer::ActiveModel {
        name: Set("Jim".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let _order = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(15.10)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert order");

    #[derive(Debug, FromQueryResult)]
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

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let _customer_jim = customer::ActiveModel {
        name: Set("Jim".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(15.10)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert order");

    let kate_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(100.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
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
        .into_iter()
        .any(|result| result.name == customer_kate.name.clone().unwrap()
            && result.order_total == Some(kate_order_1.total.clone().unwrap())));
    assert!((&results)
        .into_iter()
        .any(|result| result.name == customer_kate.name.clone().unwrap()
            && result.order_total == Some(kate_order_2.total.clone().unwrap())));

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

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(99.95)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert order");

    let kate_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(200.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
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

    assert_eq!(result.number_orders, Some(2));
    assert_eq!(
        result.total_spent,
        Some(kate_order_1.total.clone().unwrap() + kate_order_2.total.clone().unwrap())
    );
    assert_eq!(
        result.min_spent,
        Some(
            kate_order_1
                .total
                .clone()
                .unwrap()
                .min(kate_order_2.total.clone().unwrap())
        )
    );
    assert_eq!(
        result.max_spent,
        Some(
            kate_order_1
                .total
                .clone()
                .unwrap()
                .max(kate_order_2.total.clone().unwrap())
        )
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

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let customer_kate = customer::ActiveModel {
        name: Set("Kate".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let kate_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(100.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert order");

    let _kate_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_kate.id.clone().unwrap()),
        total: Set(dec!(12.00)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert order");

    let customer_bob = customer::ActiveModel {
        name: Set("Bob".to_owned()),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert customer");

    let _bob_order_1 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_bob.id.clone().unwrap()),
        total: Set(dec!(50.0)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert order");

    let _bob_order_2 = order::ActiveModel {
        bakery_id: Set(bakery.id.clone().unwrap()),
        customer_id: Set(customer_bob.id.clone().unwrap()),
        total: Set(dec!(50.0)),
        placed_at: Set(Utc::now().naive_utc()),

        ..Default::default()
    }
    .save(&ctx.db)
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
    assert_eq!(results[0].name, customer_kate.name.clone().unwrap());
    assert_eq!(
        results[0].order_total,
        Some(kate_order_1.total.clone().unwrap())
    );

    ctx.delete().await;
}
