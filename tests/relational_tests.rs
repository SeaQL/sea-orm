use sea_orm::{entity::*, query::*, FromQueryResult};

pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};

#[async_std::test]
// cargo test --test realtional_tests -- --nocapture
async fn main() {
    test_left_join().await;
}

pub async fn test_left_join() {
    let ctx = TestContext::new("mysql://root:@localhost", "test_left_join").await;

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
        bakery_id: Set(Some(bakery.id.clone().unwrap())),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert baker");

    let _baker_2 = baker::ActiveModel {
        name: Set("Baker 2".to_owned()),
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
