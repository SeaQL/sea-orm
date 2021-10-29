use super::{ColumnTrait, IdenStatic, Iterable};
use crate::{TryFromU64, TryGetableMany};
use sea_query::{FromValueTuple, IntoValueTuple};
use std::fmt::Debug;

//LINT: composite primary key cannot auto increment
/// A set of constraints to be used to define a Primary Key
pub trait PrimaryKeyTrait: IdenStatic + Iterable {
    #[allow(missing_docs)]
    type ValueType: Sized
        + Send
        + Debug
        + PartialEq
        + IntoValueTuple
        + FromValueTuple
        + TryGetableMany
        + TryFromU64;

    /// Method to call to perform `AUTOINCREMENT` operation on a Primary Kay
    fn auto_increment() -> bool;
}

/// How to map a Primary Key to a column
pub trait PrimaryKeyToColumn {
    #[allow(missing_docs)]
    type Column: ColumnTrait;

    /// Method to map a primary key to a column in an Entity
    fn into_column(self) -> Self::Column;

    /// Method to map a primary key from a column in an Entity
    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}
