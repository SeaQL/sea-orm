use super::{Column, Identity, Relation};
use sea_query::Iden;
use std::fmt::Debug;
use strum::IntoEnumIterator;

pub trait Entity: Iden + Default + Debug {
    type Model;

    type Column: Column + IntoEnumIterator;

    type Relation: Relation + IntoEnumIterator;

    fn table_name() -> Self;

    fn primary_key() -> Identity;

    fn auto_increment() -> bool {
        true
    }
}
