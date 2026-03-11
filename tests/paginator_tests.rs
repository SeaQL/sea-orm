#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{PaginatorTrait, QueryOrder, Set, entity::prelude::*};

#[sea_orm_macros::test]
async fn paginator_tests() -> Result<(), DbErr> {
    let ctx = TestContext::new("paginator_tests").await;
    create_insert_default_table(&ctx.db).await?;
    create_insert_default(&ctx.db).await?;
    paginator_num_items(&ctx.db).await?;
    paginator_num_pages(&ctx.db).await?;
    paginator_num_items_and_pages(&ctx.db).await?;
    paginator_fetch_page(&ctx.db).await?;
    paginator_count(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_insert_default(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    for _ in 0..10 {
        ActiveModel {
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    assert_eq!(
        Entity::find().all(db).await?,
        [
            Model { id: 1 },
            Model { id: 2 },
            Model { id: 3 },
            Model { id: 4 },
            Model { id: 5 },
            Model { id: 6 },
            Model { id: 7 },
            Model { id: 8 },
            Model { id: 9 },
            Model { id: 10 },
        ]
    );

    Ok(())
}

pub async fn paginator_num_items(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);
    assert_eq!(paginator.num_items().await?, 10);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 5);
    assert_eq!(paginator.num_items().await?, 10);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 10);
    assert_eq!(paginator.num_items().await?, 10);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 100);
    assert_eq!(paginator.num_items().await?, 10);

    Ok(())
}

pub async fn paginator_num_pages(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);
    assert_eq!(paginator.num_pages().await?, 4);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 5);
    assert_eq!(paginator.num_pages().await?, 2);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 10);
    assert_eq!(paginator.num_pages().await?, 1);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 7);
    assert_eq!(paginator.num_pages().await?, 2);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 100);
    assert_eq!(paginator.num_pages().await?, 1);

    Ok(())
}

pub async fn paginator_num_items_and_pages(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);

    let result = paginator.num_items_and_pages().await?;
    assert_eq!(result.number_of_items, 10);
    assert_eq!(result.number_of_pages, 4);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 5);

    let result = paginator.num_items_and_pages().await?;
    assert_eq!(result.number_of_items, 10);
    assert_eq!(result.number_of_pages, 2);

    Ok(())
}

pub async fn paginator_fetch_page(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);

    assert_eq!(
        paginator.fetch_page(0).await?,
        vec![Model { id: 1 }, Model { id: 2 }, Model { id: 3 }]
    );

    assert_eq!(
        paginator.fetch_page(1).await?,
        vec![Model { id: 4 }, Model { id: 5 }, Model { id: 6 }]
    );

    assert_eq!(
        paginator.fetch_page(2).await?,
        vec![Model { id: 7 }, Model { id: 8 }, Model { id: 9 }]
    );

    assert_eq!(paginator.fetch_page(3).await?, vec![Model { id: 10 }]);

    assert!(paginator.fetch_page(4).await?.is_empty());

    Ok(())
}

pub async fn paginator_count(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    assert_eq!(Entity::find().count(db).await?, 10);

    assert_eq!(Entity::find().filter(Column::Id.gt(5)).count(db).await?, 5);

    assert_eq!(Entity::find().filter(Column::Id.lte(3)).count(db).await?, 3);

    assert_eq!(
        Entity::find().filter(Column::Id.gt(100)).count(db).await?,
        0
    );

    Ok(())
}
