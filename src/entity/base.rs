use super::{Column, Identity, Model, Relation};
use crate::Select;
use sea_query::Iden;
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait Entity: Iden + Default + Debug {
    type Model: Model;

    type Column: Column + Iterable;

    type Relation: Relation + Iterable;

    fn primary_key() -> Identity;

    fn auto_increment() -> bool {
        true
    }

    fn find<'s>() -> Select<'s, Self> {
        Select::new(Self::default())
    }
}
