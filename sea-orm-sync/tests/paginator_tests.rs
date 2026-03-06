#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{PaginatorTrait, QueryOrder, Set, entity::prelude::*};

#[sea_orm_macros::test]
fn paginator_tests() -> Result<(), DbErr> {
    let ctx = TestContext::new("paginator_tests");
    create_insert_default_table(&ctx.db)?;
    create_insert_default(&ctx.db)?;
    paginator_num_items(&ctx.db)?;
    paginator_num_pages(&ctx.db)?;
    paginator_num_items_and_pages(&ctx.db)?;
    paginator_fetch_page(&ctx.db)?;
    paginator_count(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn create_insert_default(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    for _ in 0..10 {
        ActiveModel {
            ..Default::default()
        }
        .insert(db)?;
    }

    assert_eq!(
        Entity::find().all(db)?,
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

pub fn paginator_num_items(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);
    assert_eq!(paginator.num_items()?, 10);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 5);
    assert_eq!(paginator.num_items()?, 10);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 10);
    assert_eq!(paginator.num_items()?, 10);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 100);
    assert_eq!(paginator.num_items()?, 10);

    Ok(())
}

pub fn paginator_num_pages(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);
    assert_eq!(paginator.num_pages()?, 4);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 5);
    assert_eq!(paginator.num_pages()?, 2);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 10);
    assert_eq!(paginator.num_pages()?, 1);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 7);
    assert_eq!(paginator.num_pages()?, 2);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 100);
    assert_eq!(paginator.num_pages()?, 1);

    Ok(())
}

pub fn paginator_num_items_and_pages(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);

    let result = paginator.num_items_and_pages()?;
    assert_eq!(result.number_of_items, 10);
    assert_eq!(result.number_of_pages, 4);

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 5);

    let result = paginator.num_items_and_pages()?;
    assert_eq!(result.number_of_items, 10);
    assert_eq!(result.number_of_pages, 2);

    Ok(())
}

pub fn paginator_fetch_page(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    let paginator = Entity::find().order_by_asc(Column::Id).paginate(db, 3);

    assert_eq!(
        paginator.fetch_page(0)?,
        vec![Model { id: 1 }, Model { id: 2 }, Model { id: 3 }]
    );

    assert_eq!(
        paginator.fetch_page(1)?,
        vec![Model { id: 4 }, Model { id: 5 }, Model { id: 6 }]
    );

    assert_eq!(
        paginator.fetch_page(2)?,
        vec![Model { id: 7 }, Model { id: 8 }, Model { id: 9 }]
    );

    assert_eq!(paginator.fetch_page(3)?, vec![Model { id: 10 }]);

    assert!(paginator.fetch_page(4)?.is_empty());

    Ok(())
}

pub fn paginator_count(db: &DatabaseConnection) -> Result<(), DbErr> {
    use insert_default::*;

    assert_eq!(Entity::find().count(db)?, 10);

    assert_eq!(Entity::find().filter(Column::Id.gt(5)).count(db)?, 5);

    assert_eq!(Entity::find().filter(Column::Id.lte(3)).count(db)?, 3);

    assert_eq!(Entity::find().filter(Column::Id.gt(100)).count(db)?, 0);

    Ok(())
}
