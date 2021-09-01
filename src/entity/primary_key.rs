use super::{ColumnTrait, IdenStatic, Iterable};
use crate::{TryFromU64, TryGetableMany};
use sea_query::IntoValueTuple;
use std::fmt::Debug;

//LINT: composite primary key cannot auto increment
pub trait PrimaryKeyTrait: IdenStatic + Iterable {
    type ValueType: Sized
        + Default
        + Debug
        + PartialEq
        + IntoValueTuple
        + TryGetableMany
        + TryFromU64;

    fn auto_increment() -> bool;
}

pub trait PrimaryKeyToColumn {
    type Column: ColumnTrait;

    fn into_column(self) -> Self::Column;

    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}
