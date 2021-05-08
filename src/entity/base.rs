use super::{ColumnTrait, Identity, ModelTrait, RelationTrait};
use crate::Select;
use sea_query::Iden;
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
}
