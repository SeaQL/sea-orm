use super::SelectorTrait;
use crate::{ConnectionTrait, StatementBuilder, error::*};
use itertools::Itertools;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub(super) struct ReturningSelector<S, Q>
where
    S: SelectorTrait,
    Q: StatementBuilder,
{
    pub(crate) query: Q,
    selector: PhantomData<S>,
}

impl<S, Q> ReturningSelector<S, Q>
where
    S: SelectorTrait,
    Q: StatementBuilder,
{
    pub fn from_query(query: Q) -> Self {
        Self {
            query,
            selector: PhantomData,
        }
    }

    pub fn one<C>(self, db: &C) -> Result<Option<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let row = db.query_one(&self.query)?;
        match row {
            Some(row) => Ok(Some(S::from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    pub fn all<C>(self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        db.query_all(&self.query)?
            .into_iter()
            .map(|row| S::from_raw_query_result(row))
            .try_collect()
    }
}
