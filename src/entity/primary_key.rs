use sea_query::Value;

use crate::TryGetable;

use super::{ColumnTrait, IdenStatic, Iterable};

//LINT: composite primary key cannot auto increment
pub trait PrimaryKeyTrait: IdenStatic + Iterable {
    type ValueType: TryGetable + Into<Value> + Clone + std::fmt::Debug;

    fn auto_increment() -> bool;
}

pub trait PrimaryKeyToColumn {
    type Column: ColumnTrait;

    fn into_column(self) -> Self::Column;

    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}
