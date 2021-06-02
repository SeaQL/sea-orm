use super::{EntityTrait, IdenStatic, Iterable};

pub trait PrimaryKeyTrait: IdenStatic + Iterable {}

pub trait PrimaryKeyToColumn<E>
where
    E: EntityTrait,
{
    fn into_column(self) -> E::Column;

    fn from_column(col: E::Column) -> Option<Self> where Self: std::marker::Sized;
}
