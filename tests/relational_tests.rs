use sea_orm::{entity::*, InsertResult};

// pub mod bakery_chain;
// pub use bakery_chain::*;

pub mod common;
pub use common::{setup::*, TestContext};

#[async_std::test]
// cargo test --test realtional_tests -- --nocapture
async fn main() {
    test_left_join().await;
}

pub async fn test_left_join() {
    let ctx = TestContext::new("test", function!()).await;

    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let res: InsertResult = Bakery::insert(seaside_bakery)
        .exec(&ctx.db)
        .await
        .expect("could not insert bakery");

    let bakery: Option<bakery::Model> = Bakery::find_by_id(res.last_insert_id)
        .one(&ctx.db)
        .await
        .expect("could not find bakery");

    // assert!(bakery.is_some());
    // let bakery_model = bakery.unwrap();
    // assert_eq!(bakery_model.name, "SeaSide Bakery");
    // assert_eq!(bakery_model.profit_margin, 10.4);
}
