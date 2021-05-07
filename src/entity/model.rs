use crate::{QueryResult, TypeErr};

pub trait Model {
    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr>
    where
        Self: std::marker::Sized;
}
