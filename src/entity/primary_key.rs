use super::{IdenStatic, Iterable, ModelTrait};

pub trait PrimaryKeyTrait: IdenStatic + Iterable {}

pub trait PrimaryKeyOfModel<M>
where
    M: ModelTrait,
{
    fn into_column(self) -> M::Column;
}
