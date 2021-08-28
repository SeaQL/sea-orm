use super::{ColumnTrait, IdenStatic, Iterable};

//LINT: composite primary key cannot auto increment
pub trait PrimaryKeyTrait: IdenStatic + Iterable {
    type ValueType: Sized;

    fn auto_increment() -> bool;
}

pub trait PrimaryKeyToColumn {
    type Column: ColumnTrait;

    fn into_column(self) -> Self::Column;

    fn from_column(col: Self::Column) -> Option<Self>
    where
        Self: Sized;
}
