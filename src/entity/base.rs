use super::{Column, Identity, Model, Relation};
use crate::Select;
use sea_query::Iden;
use std::fmt::Debug;
use strum::IntoEnumIterator;

pub trait Entity: Iden + Default + Debug {
    type Model: Model;

    type Column: Column + IntoEnumIterator;

    type Relation: Relation + IntoEnumIterator;

    fn primary_key() -> Identity;

    fn auto_increment() -> bool {
        true
    }

    fn find<'s>() -> Select<'s, Self> {
        Select::new(Self::default())
    }
}
