// cargo test --test query_tests -- --nocapture
use sea_orm::entity::*;
use sea_orm::QueryFilter;

pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]

pub async fn find_one_with_no_result() {
    let ctx = TestContext::new("find_one_with_no_result").await;

    let bakery = Bakery::find().one(&ctx.db).await.unwrap();
    assert_eq!(bakery, None);

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_one_with_result() {
    let ctx = TestContext::new("find_one_with_result").await;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let result = Bakery::find().one(&ctx.db).await.unwrap().unwrap();

    assert_eq!(result.id, bakery.id.unwrap());

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_by_id_with_no_result() {
    let ctx = TestContext::new("find_by_id_with_no_result").await;

    let bakery = Bakery::find_by_id(999).one(&ctx.db).await.unwrap();
    assert_eq!(bakery, None);

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_by_id_with_result() {
    let ctx = TestContext::new("find_by_id_with_result").await;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let result = Bakery::find_by_id(bakery.id.clone().unwrap())
        .one(&ctx.db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(result.id, bakery.id.unwrap());

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_all_with_no_result() {
    let ctx = TestContext::new("find_all_with_no_result").await;

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 0);

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_all_with_result() {
    let ctx = TestContext::new("find_all_with_result").await;

    let _ = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let _ = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();

    assert_eq!(bakeries.len(), 2);

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_all_filter_no_result() {
    let ctx = TestContext::new("find_all_filter_no_result").await;

    let _ = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let _ = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("Good"))
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(bakeries.len(), 0);

    ctx.delete().await;
}

#[async_std::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
#[cfg(feature = "sqlx-mysql")]
pub async fn find_all_filter_with_results() {
    let ctx = TestContext::new("find_all_filter_with_results").await;

    let _ = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let _ = bakery::ActiveModel {
        name: Set("Top Bakery".to_owned()),
        profit_margin: Set(15.0),
        ..Default::default()
    }
    .save(&ctx.db)
    .await
    .expect("could not insert bakery");

    let bakeries = Bakery::find()
        .filter(bakery::Column::Name.contains("bakery"))
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(bakeries.len(), 2);

    ctx.delete().await;
}
