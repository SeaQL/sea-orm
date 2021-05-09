use super::{IdenStatic, ModelTrait};

pub trait PrimaryKeyTrait: IdenStatic {}

pub trait PrimaryKeyOfModel<M>
where
    M: ModelTrait,
{
    fn into_column(self) -> M::Column;
}
