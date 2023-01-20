use crate::EntityTrait;
use sea_query::TableTruncateStatement;
use std::marker::PhantomData;

/// A helper to truncate a table
#[derive(Debug)]
pub struct Truncate<E>
where
    E: EntityTrait,
{
    pub(crate) query: TableTruncateStatement,
    pub(crate) entity: PhantomData<E>,
}

impl<E> Default for Truncate<E>
where
    E: EntityTrait,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E> Truncate<E>
where
    E: EntityTrait,
{
    /// Construct a truncate helper
    pub fn new() -> Self {
        Self {
            query: TableTruncateStatement::new()
                .table(E::default().table_ref())
                .to_owned(),
            entity: PhantomData,
        }
    }
}
