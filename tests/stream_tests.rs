pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
pub use sea_orm::entity::*;
pub use sea_orm::{ConnectionTrait, DbErr, QueryFilter, QueryOrder};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
pub async fn stream() -> Result<(), DbErr> {
    use futures::StreamExt;

    let ctx = TestContext::new("stream").await;
    create_tables(&ctx.db).await?;

    let bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    }
    .save(&ctx.db)
    .await?;

    {
        let result = Bakery::find()
            .filter(bakery::Column::Name.contains("a"))
            .order_by_asc(bakery::Column::Name)
            .stream(&ctx.db)
            .await?;
    }

    let result = Bakery::find()
        .filter(bakery::Column::Name.contains("a"))
        .order_by_asc(bakery::Column::Name)
        .stream(&ctx.db)
        .await?;

    println!("--- This line is visible ---");

    // Error: Query("Failed to acquire connection from pool.")
    let result = Bakery::find_by_id(bakery.id.clone().unwrap())
        .stream(&ctx.db)
        .await?
        .next()
        .await
        .unwrap()?;
    
    println!("--- Not visible ---");

    assert_eq!(result.id, bakery.id.unwrap());

    ctx.delete().await;

    Ok(())
}
