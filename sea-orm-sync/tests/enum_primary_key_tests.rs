#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{
    ActiveEnum as ActiveEnumTrait, DatabaseConnection,
    entity::prelude::*,
    entity::*,
    sea_query::{BinOper, Expr},
};
use sea_query::ExprTrait;

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("enum_primary_key_tests");
    create_tables(&ctx.db)?;
    insert_teas(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn insert_teas(db: &DatabaseConnection) -> Result<(), DbErr> {
    use teas::*;

    let model = Model {
        id: Tea::EverydayTea,
        category: None,
        color: None,
    };

    assert_eq!(
        model,
        ActiveModel {
            id: Set(Tea::EverydayTea),
            category: Set(None),
            color: Set(None),
        }
        .insert(db)?
    );
    assert_eq!(model, Entity::find().one(db)?.unwrap());
    assert_eq!(
        model,
        Entity::find()
            .filter(Column::Id.is_not_null())
            .filter(Column::Category.is_null())
            .filter(Column::Color.is_null())
            .one(db)?
            .unwrap()
    );

    // UNIQUE constraint failed
    assert!(
        ActiveModel {
            id: Set(Tea::EverydayTea),
            category: Set(Some(Category::Big)),
            color: Set(Some(Color::Black)),
        }
        .insert(db)
        .is_err()
    );

    // UNIQUE constraint failed
    assert!(
        Entity::insert(ActiveModel {
            id: Set(Tea::EverydayTea),
            category: Set(Some(Category::Big)),
            color: Set(Some(Color::Black)),
        })
        .exec(db)
        .is_err()
    );

    let _ = ActiveModel {
        category: Set(Some(Category::Big)),
        color: Set(Some(Color::Black)),
        ..model.into_active_model()
    }
    .save(db)?;

    let model = Entity::find().one(db)?.unwrap();
    assert_eq!(
        model,
        Model {
            id: Tea::EverydayTea,
            category: Some(Category::Big),
            color: Some(Color::Black),
        }
    );
    assert_eq!(
        model,
        Entity::find()
            .filter(Column::Id.eq(Tea::EverydayTea))
            .filter(Column::Category.eq(Category::Big))
            .filter(Column::Color.eq(Color::Black))
            .one(db)?
            .unwrap()
    );
    assert_eq!(
        model,
        Entity::find()
            .filter(Expr::col(Column::Id).binary(
                BinOper::In,
                Expr::tuple([ActiveEnumTrait::as_enum(&Tea::EverydayTea)])
            ))
            .one(db)?
            .unwrap()
    );
    // Equivalent to the above.
    assert_eq!(
        model,
        Entity::find()
            .filter(Column::Id.is_in([Tea::EverydayTea]))
            .one(db)?
            .unwrap()
    );

    let res = model.delete(db)?;

    assert_eq!(res.rows_affected, 1);
    assert_eq!(Entity::find().one(db)?, None);

    Ok(())
}
