use super::{ModelTrait, IdenStatic};

pub trait PrimaryKeyTrait: IdenStatic {}

pub trait PrimaryKeyOfModel<M>
where
    M: ModelTrait,
{
    fn into_column(self) -> M::Column;
}