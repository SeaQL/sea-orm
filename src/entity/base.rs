use super::{ColumnTrait, Identity, ModelTrait, RelationTrait};
use crate::Select;
use sea_query::{Expr, Iden, Value};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait EntityTrait: Iden + Default + Debug {
    type Model: ModelTrait;

    type Column: ColumnTrait + Iterable;

    type Relation: RelationTrait + Iterable;

    fn primary_key() -> Identity;

    fn auto_increment() -> bool {
        true
    }

    fn find<'s>() -> Select<'s, Self> {
        Select::new(Self::default())
    }

    fn find_one<'s>() -> Select<'s, Self> {
        let mut select = Self::find();
        select.query().limit(1);
        select
    }

    fn find_one_by<'s, V>(v: V) -> Select<'s, Self>
    where
        V: Into<Value>,
    {
        let select = Self::find_one();
        let select =
            select.filter(Expr::tbl(Self::default(), Self::primary_key().into_iden()).eq(v));
        select
    }
}
