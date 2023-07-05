use std::ops::Deref;

use entity::{Column, Entity};
use sea_orm::{ColumnTrait, DerivePartialModel, FromQueryResult, TryGetable};
use sea_query::Expr;

mod entity {
    use sea_orm::prelude::*;

    #[derive(Debug, Clone, DeriveEntityModel)]
    #[sea_orm(table_name = "foo_table")]
    pub struct Model {
        #[sea_orm(primary_key)]
        id: i32,
        foo: i32,
        bar: String,
        foo2: bool,
        bar2: f64,
    }

    #[derive(Debug, DeriveRelation, EnumIter)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct SimpleTest {
    _foo: i32,
    _bar: String,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct FieldFromDiffNameColumnTest {
    #[sea_orm(from_col = "foo2")]
    _foo: i32,
    #[sea_orm(from_col = "bar2")]
    _bar: String,
}

#[derive(FromQueryResult, DerivePartialModel)]
struct FieldFromExpr {
    #[sea_orm(from_expr = "Column::Bar2.sum()")]
    _foo: f64,
    #[sea_orm(from_expr = "Expr::col(Column::Id).equals(Column::Foo)")]
    _bar: bool,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct GenericTest<T>
where
    T: TryGetable,
{
    _foo: i32,
    _bar: T,
}
#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct MultiGenericTest<T: TryGetable, F: TryGetable> {
    #[sea_orm(from_expr = "Column::Bar2.sum()")]
    _foo: T,
    _bar: F,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct GenericWithBoundsTest<T: TryGetable + Copy + Clone + 'static> {
    _foo: T,
}

#[derive(FromQueryResult, DerivePartialModel)]
#[sea_orm(entity = "Entity")]
struct WhereGenericTest<T>
where
    T: TryGetable + Deref,
    <T as Deref>::Target: Clone,
{
    _foo: T,
}

#[derive(FromQueryResult, DerivePartialModel)]
struct MixedBoundTest<T: TryGetable + Clone, F>
where
    F: TryGetable + Clone,
{
    #[sea_orm(from_expr = "Column::Bar2.sum()")]
    _foo: T,
    #[sea_orm(from_expr = "Expr::col(Column::Id).equals(Column::Foo)")]
    _bar: F,
}
