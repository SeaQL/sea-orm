#![allow(missing_docs)]
use super::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait};
use crate::{
    ConnectionTrait, DbErr, IntoSimpleExpr, ItemsAndPagesNumber, Iterable, ModelTrait, QueryFilter,
    QueryOrder,
};
use sea_query::{IntoValueTuple, Order, TableRef};
use std::marker::PhantomData;

mod has_many;
mod has_one;

pub use has_many::{HasMany, Iter as HasManyIter};
pub use has_one::HasOne;

pub trait EntityLoaderTrait<E: EntityTrait>: QueryFilter + QueryOrder + Clone {
    /// The return type of this loader
    type ModelEx: ModelTrait<Entity = E>;

    /// Find a model by primary key
    fn filter_by_id<T>(mut self, values: T) -> Self
    where
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let mut keys = E::PrimaryKey::iter();
        for v in values.into().into_value_tuple() {
            if let Some(key) = keys.next() {
                let col = key.into_column();
                self.filter_mut(col.eq(v));
            } else {
                unreachable!("primary key arity mismatch");
            }
        }
        self
    }

    /// Apply order by primary key to the query statement
    fn order_by_id_asc(self) -> Self {
        self.order_by_id(Order::Asc)
    }

    /// Apply order by primary key to the query statement
    fn order_by_id_desc(self) -> Self {
        self.order_by_id(Order::Desc)
    }

    /// Apply order by primary key to the query statement
    fn order_by_id(mut self, order: Order) -> Self {
        for key in E::PrimaryKey::iter() {
            let col = key.into_column();
            <Self as QueryOrder>::query(&mut self)
                .order_by_expr(col.into_simple_expr(), order.clone());
        }
        self
    }

    /// Paginate query.
    fn paginate<'db, C: ConnectionTrait>(
        self,
        db: &'db C,
        page_size: u64,
    ) -> EntityLoaderPaginator<'db, C, E, Self> {
        EntityLoaderPaginator {
            loader: self,
            page: 0,
            page_size,
            db,
            phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    fn fetch<C: ConnectionTrait>(
        self,
        db: &C,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<Self::ModelEx>, DbErr>;

    #[doc(hidden)]
    fn num_items<C: ConnectionTrait>(self, db: &C, page_size: u64) -> Result<u64, DbErr>;
}

#[derive(Debug)]
pub struct EntityLoaderPaginator<'db, C, E, L>
where
    C: ConnectionTrait,
    E: EntityTrait,
    L: EntityLoaderTrait<E>,
{
    pub(crate) loader: L,
    pub(crate) page: u64,
    pub(crate) page_size: u64,
    pub(crate) db: &'db C,
    pub(crate) phantom: PhantomData<E>,
}

/// Just a marker trait on EntityReverse
pub trait EntityReverse {
    type Entity: EntityTrait;
}

/// Subject to change, not yet stable
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EntityLoaderWithSelf<R: EntityTrait, S: EntityTrait>(pub R, pub S);

/// Subject to change, not yet stable
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EntityLoaderWithSelfRev<R: EntityTrait, S: EntityReverse>(pub R, pub S);

#[derive(Debug, Clone, PartialEq)]
pub enum LoadTarget {
    TableRef(TableRef),
    TableRefRev(TableRef),
    Relation(String),
}

impl<'db, C, E, L> EntityLoaderPaginator<'db, C, E, L>
where
    C: ConnectionTrait,
    E: EntityTrait,
    L: EntityLoaderTrait<E>,
{
    /// Fetch a specific page; page index starts from zero
    pub fn fetch_page(&self, page: u64) -> Result<Vec<L::ModelEx>, DbErr> {
        self.loader.clone().fetch(self.db, page, self.page_size)
    }

    /// Fetch the current page
    pub fn fetch(&self) -> Result<Vec<L::ModelEx>, DbErr> {
        self.fetch_page(self.page)
    }

    /// Get the total number of items
    pub fn num_items(&self) -> Result<u64, DbErr> {
        self.loader.clone().num_items(self.db, self.page_size)
    }

    /// Get the total number of pages
    pub fn num_pages(&self) -> Result<u64, DbErr> {
        let num_items = self.num_items()?;
        let num_pages = self.compute_pages_number(num_items);
        Ok(num_pages)
    }

    /// Get the total number of items and pages
    pub fn num_items_and_pages(&self) -> Result<ItemsAndPagesNumber, DbErr> {
        let number_of_items = self.num_items()?;
        let number_of_pages = self.compute_pages_number(number_of_items);

        Ok(ItemsAndPagesNumber {
            number_of_items,
            number_of_pages,
        })
    }

    /// Compute the number of pages for the current page
    fn compute_pages_number(&self, num_items: u64) -> u64 {
        (num_items / self.page_size) + (num_items % self.page_size > 0) as u64
    }

    /// Increment the page counter
    pub fn next(&mut self) {
        self.page += 1;
    }

    /// Get current page number
    pub fn cur_page(&self) -> u64 {
        self.page
    }

    /// Fetch one page and increment the page counter
    pub fn fetch_and_next(&mut self) -> Result<Option<Vec<L::ModelEx>>, DbErr> {
        let vec = self.fetch()?;
        self.next();
        let opt = if !vec.is_empty() { Some(vec) } else { None };
        Ok(opt)
    }
}

#[cfg(test)]
mod test {
    use crate::ModelTrait;
    use crate::tests_cfg::cake;

    #[test]
    fn test_model_ex_convert() {
        let cake = cake::Model {
            id: 12,
            name: "hello".into(),
        };
        let cake_ex: cake::ModelEx = cake.clone().into();

        assert_eq!(cake, cake_ex);
        assert_eq!(cake_ex, cake);
        assert_eq!(cake.id, cake_ex.id);
        assert_eq!(cake.name, cake_ex.name);

        assert_eq!(cake_ex.get(cake::Column::Id), 12i32.into());
        assert_eq!(cake_ex.get(cake::Column::Name), "hello".into());

        assert_eq!(cake::Model::from(cake_ex), cake);
    }
}
