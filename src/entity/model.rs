use crate::{ColumnTrait, QueryResult, TypeErr};
pub use sea_query::Value;
use std::fmt::Debug;

pub trait ModelTrait: Clone + Debug + Default {
    type Column: ColumnTrait;

    fn get(&self, c: Self::Column) -> Value;

    fn set(&mut self, c: Self::Column, v: Value);

    fn from_query_result(row: QueryResult, pre: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

pub trait FromQueryResult {
    fn from_query_result(row: QueryResult, pre: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

impl<M> FromQueryResult for M
where
    M: ModelTrait + Sized,
{
    fn from_query_result(row: QueryResult, pre: &str) -> Result<M, TypeErr> {
        <Self as ModelTrait>::from_query_result(row, pre)
    }
}
