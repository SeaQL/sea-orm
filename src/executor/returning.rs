use super::SelectorTrait;
use crate::{
    ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, Iterable, QueryResult, StatementBuilder,
    error::*,
};
use futures_util::FutureExt;
use itertools::Itertools;
use sea_query::{Query, ReturningClause};
use std::{future::Future, marker::PhantomData};

pub(super) fn returning_all_columns<E: EntityTrait>(backend: DbBackend) -> ReturningClause {
    Query::returning()
        .exprs(E::Column::iter().map(|col| col.select_as(col.into_returning_expr(backend))))
}

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

    pub fn one<C>(self, db: &C) -> impl Future<Output = Result<Option<S::Item>, DbErr>>
    where
        C: ConnectionTrait,
    {
        let task = db.query_one(&self.query);
        let convert = |result: Result<Option<QueryResult>, DbErr>| {
            let row = result?;
            row.map(S::from_raw_query_result).transpose()
        };
        #[cfg(not(feature = "sync"))]
        {
            FutureExt::map(task, convert)
        }
        #[cfg(feature = "sync")]
        convert(task)
    }

    pub fn all<C>(self, db: &C) -> impl Future<Output = Result<Vec<S::Item>, DbErr>>
    where
        C: ConnectionTrait,
    {
        let task = db.query_all(&self.query);
        let convert = |result: Result<Vec<QueryResult>, DbErr>| {
            let rows = result?;
            rows.into_iter().map(S::from_raw_query_result).try_collect()
        };
        #[cfg(not(feature = "sync"))]
        {
            FutureExt::map(task, convert)
        }
        #[cfg(feature = "sync")]
        convert(task)
    }
}
