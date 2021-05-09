use crate::{ColumnTrait, QueryResult, TypeErr};
pub use sea_query::Value;
use std::fmt::Debug;

pub trait ModelTrait: Clone + Debug + Default {
    type Column: ColumnTrait;

    fn get(&self, c: Self::Column) -> Value;

    fn set(&mut self, c: Self::Column, v: Value);

    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr>
    where
        Self: std::marker::Sized;
}
