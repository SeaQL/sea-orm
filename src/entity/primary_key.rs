use super::{ColumnTrait, IdenStatic, Iterable};
use crate::{ActiveModelTrait, EntityTrait, TryFromU64, TryGetableMany};
use sea_query::IntoValueTuple;
use std::fmt::Debug;

//LINT: composite primary key cannot auto increment
pub trait PrimaryKeyTrait: IdenStatic + Iterable {
    type ValueType: Sized + Send + Debug + PartialEq + IntoValueTuple + TryGetableMany + TryFromU64;

    fn auto_increment() -> bool;
}

pub trait PrimaryKeyToColumn {
    type Column: ColumnTrait;

    fn into_column(self) -> Self::Column;

    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}

pub trait PrimaryKeyValue<E>
where
    E: EntityTrait,
{
    fn get_primary_key_value<A>(active_model: A) -> <E::PrimaryKey as PrimaryKeyTrait>::ValueType
    where
        A: ActiveModelTrait<Entity = E>;
}
