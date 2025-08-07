use super::SelectorTrait;
use crate::{ConnectionTrait, StatementBuilder, error::*};
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

    pub async fn one<C>(self, db: &C) -> Result<Option<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let row = db.query_one(&self.query).await?;
        match row {
            Some(row) => Ok(Some(S::from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    pub async fn all<C>(self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let rows = db.query_all(&self.query).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(S::from_raw_query_result(row)?);
        }
        Ok(models)
    }
}
