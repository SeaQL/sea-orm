#![allow(unused_imports, dead_code)]

pub mod common;

pub use common::{TestContext, features::*, setup::*};
use pretty_assertions::assert_eq;
use sea_orm::{
    DatabaseConnection, Delete, IntoActiveModel, Iterable, QueryTrait, Set, Update,
    entity::prelude::*,
};
use sea_query::{Expr, ExprTrait, Query};

#[sea_orm_macros::test]
fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("dyn_table_name_tests");
    create_dyn_table_name_lazy_static_table(&ctx.db)?;
    dyn_table_name(&ctx.db)?;
    ctx.delete();

    Ok(())
}

pub fn dyn_table_name(db: &DatabaseConnection) -> Result<(), DbErr> {
    use dyn_table_name::*;

    for i in 1..=2 {
        let entity = Entity {
            table_name: i as u32,
        };
        let model = Model {
            id: 1,
            name: "1st Row".into(),
        };
        // Prepare insert statement
        let mut insert = Entity::insert(model.clone().into_active_model());
        // Reset the table name of insert statement
        insert.query().into_table(entity.table_ref());
        // Execute the insert statement
        assert_eq!(insert.exec(db)?.last_insert_id, 1);

        // Prepare select statement
        let mut select = Entity::find();
        // Override the select statement
        *QueryTrait::query(&mut select) = Query::select()
            .exprs(Column::iter().map(|col| col.select_as(Expr::col(col))))
            .from(entity.table_ref())
            .to_owned();
        // Execute the select statement
        assert_eq!(select.clone().one(db)?, Some(model.clone()));

        // Prepare update statement
        let update = Update::many(entity).set(ActiveModel {
            name: Set("1st Row (edited)".into()),
            ..model.clone().into_active_model()
        });
        // Execute the update statement
        assert_eq!(update.exec(db)?.rows_affected, 1);

        assert_eq!(
            select.clone().one(db)?,
            Some(Model {
                id: 1,
                name: "1st Row (edited)".into(),
            })
        );

        // Prepare delete statement
        let delete = Delete::many(entity).filter(Expr::col(Column::Id).eq(1));
        // Execute the delete statement
        assert_eq!(delete.exec(db)?.rows_affected, 1);
        assert_eq!(select.one(db)?, None);
    }

    Ok(())
}
