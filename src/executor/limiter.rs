use crate::{
    ConnectionTrait, DbErr, EntityTrait, FromQueryResult, Paginator, PaginatorTrait, QuerySelect,
    Select, SelectModel, Selector, SelectorTrait,
};
use std::num::NonZeroU64;

#[derive(Debug)]
pub struct Limiter<'db, C, S>
where
    C: ConnectionTrait,
    S: SelectorTrait + 'db,
{
    db: &'db C,
    selector: Selector<S>,
    paginator: Paginator<'db, C, S>,
}

impl<'db, C, S> Limiter<'db, C, S>
where
    C: ConnectionTrait,
    S: SelectorTrait + 'db,
{
    pub async fn fetch(self) -> Result<Vec<S::Item>, DbErr> {
        self.selector.all(self.db).await
    }

    pub async fn total(&self) -> Result<u64, DbErr> {
        self.paginator.num_items().await
    }
}

pub trait LimiterTrait<'db, C>
where
    C: ConnectionTrait,
{
    type Selector: SelectorTrait + 'db;

    fn limiting(self, db: &'db C, offset: u64, limit: u64) -> Limiter<'db, C, Self::Selector>;
}

impl<'db, C, M, E> LimiterTrait<'db, C> for Select<E>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync + 'db,
{
    type Selector = SelectModel<M>;

    fn limiting(self, db: &'db C, offset: u64, limit: u64) -> Limiter<'db, C, Self::Selector> {
        let selector = self
            .clone()
            .limit(NonZeroU64::new(limit).map(|limit| limit.get()))
            .offset(NonZeroU64::new(limit).map(|limit| limit.get()))
            .into_model();

        Limiter {
            db,
            paginator: self.clone().paginate(db, 1),
            selector,
        }
    }
}
