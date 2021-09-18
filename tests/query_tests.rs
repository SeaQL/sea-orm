pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use sea_orm::{DatabaseTransaction, DbErr};
pub use sea_orm::entity::*;
pub use sea_orm::{QueryFilter, ConnectionTrait};

// Run the test locally:
// DATABASE_URL="mysql://root:@localhost" cargo test --features sqlx-mysql,runtime-async-std --test query_tests
#[sea_orm_macros::test]
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

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
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

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn find_by_id_with_no_result() {
    let ctx = TestContext::new("find_by_id_with_no_result").await;

    let bakery = Bakery::find_by_id(999).one(&ctx.db).await.unwrap();
    assert_eq!(bakery, None);

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
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

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn find_all_with_no_result() {
    let ctx = TestContext::new("find_all_with_no_result").await;

    let bakeries = Bakery::find().all(&ctx.db).await.unwrap();
    assert_eq!(bakeries.len(), 0);

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
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

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
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

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
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
        .filter(bakery::Column::Name.contains("Bakery"))
        .all(&ctx.db)
        .await
        .unwrap();

    assert_eq!(bakeries.len(), 2);

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn transaction() {
    let ctx = TestContext::new("transaction_test").await;

    ctx.db.transaction::<_, (), DbErr>(|txn| Box::pin(async move {
        let _ = bakery::ActiveModel {
            name: Set("SeaSide Bakery".to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let _ = bakery::ActiveModel {
            name: Set("Top Bakery".to_owned()),
            profit_margin: Set(15.0),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains("Bakery"))
            .all(txn)
            .await?;

        assert_eq!(bakeries.len(), 2);

        Ok(())
    })).await.unwrap();

    ctx.delete().await;
}

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn transaction_with_reference() {
    let ctx = TestContext::new("transaction_with_reference_test").await;
    let name1 = "SeaSide Bakery";
    let name2 = "Top Bakery";
    let search_name = "Bakery";
    ctx.db.transaction(|txn| _transaction_with_reference(txn, name1, name2, search_name)).await.unwrap();

    ctx.delete().await;
}

fn _transaction_with_reference<'a>(txn: &'a DatabaseTransaction, name1: &'a str, name2: &'a str, search_name: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(), DbErr>> + Send + 'a>> {
    Box::pin(async move {
        let _ = bakery::ActiveModel {
            name: Set(name1.to_owned()),
            profit_margin: Set(10.4),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let _ = bakery::ActiveModel {
            name: Set(name2.to_owned()),
            profit_margin: Set(15.0),
            ..Default::default()
        }
            .save(txn)
            .await?;

        let bakeries = Bakery::find()
            .filter(bakery::Column::Name.contains(search_name))
            .all(txn)
            .await?;

        assert_eq!(bakeries.len(), 2);

        Ok(())
    })
}
