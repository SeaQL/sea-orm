use super::{IdenStatic, Iterable, EntityTrait};

pub trait PrimaryKeyTrait: IdenStatic + Iterable {}

pub trait PrimaryKeyToColumn<E>
where
    E: EntityTrait,
{
    fn into_column(self) -> E::Column;
}
