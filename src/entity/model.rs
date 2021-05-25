use crate::{ColumnTrait, QueryResult, TypeErr};
pub use sea_query::Value;
use std::fmt::Debug;
use serde::Serialize;

// TODO: Any better way to do so?
#[cfg(feature = "serialize-query-result")]
pub trait ModelTrait: Serialize + Clone + Debug {
    type Column: ColumnTrait;

    fn get(&self, c: Self::Column) -> Value;

    fn set(&mut self, c: Self::Column, v: Value);

    fn from_query_result(row: &QueryResult, pre: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

// TODO: Any better way to do so?
#[cfg(not(feature = "serialize-query-result"))]
pub trait ModelTrait: Clone + Debug {
    type Column: ColumnTrait;

    fn get(&self, c: Self::Column) -> Value;

    fn set(&mut self, c: Self::Column, v: Value);

    fn from_query_result(row: &QueryResult, pre: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

pub trait FromQueryResult {
    fn from_query_result(row: &QueryResult, pre: &str) -> Result<Self, TypeErr>
    where
        Self: Sized;
}

impl<M> FromQueryResult for M
where
    M: ModelTrait + Sized,
{
    fn from_query_result(row: &QueryResult, pre: &str) -> Result<M, TypeErr> {
        <Self as ModelTrait>::from_query_result(row, pre)
    }
}
