use super::{ColumnTrait, IdenStatic, Iterable};
use crate::TryGetable;
use sea_query::IntoValueTuple;
use std::fmt::{Debug, Display};

//LINT: composite primary key cannot auto increment
pub trait PrimaryKeyTrait: IdenStatic + Iterable {
    type ValueType: Sized + Default + Debug + Display + PartialEq + IntoValueTuple + TryGetable;

    fn auto_increment() -> bool;
}

pub trait PrimaryKeyToColumn {
    type Column: ColumnTrait;

    fn into_column(self) -> Self::Column;

    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}
