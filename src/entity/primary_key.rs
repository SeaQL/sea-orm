use super::{ColumnTrait, IdenStatic, Iterable};

pub trait PrimaryKeyTrait: IdenStatic + Iterable {}

pub trait PrimaryKeyToColumn {
    type Column: ColumnTrait;

    fn into_column(self) -> Self::Column;

    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}
