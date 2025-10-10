#![allow(missing_docs)]
use super::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait};
use crate::{Iterable, QueryFilter};
use sea_query::IntoValueTuple;
use std::marker::PhantomData;

pub trait EntityLoaderTrait<E: EntityTrait>: QueryFilter {
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
}

#[derive(Debug, Clone)]
pub struct HasOne<E: EntityTrait> {
    phantom: PhantomData<E>,
    pub(crate) item: Option<Box<E::Model>>,
}

pub type BelongsTo<E> = HasOne<E>;

// TODO impl serde

impl<E: EntityTrait> Default for HasOne<E> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            item: None,
        }
    }
}

impl<E: EntityTrait> HasOne<E> {
    pub fn get(&self) -> Option<&E::Model> {
        self.item.as_deref()
    }

    pub fn set(&mut self, item: Option<Box<E::Model>>) {
        self.item = item
    }
}

#[derive(Debug, Clone)]
pub struct HasMany<E: EntityTrait> {
    phantom: PhantomData<E>,
    pub(crate) items: Vec<E::Model>,
}

impl<E: EntityTrait> Default for HasMany<E> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            items: Default::default(),
        }
    }
}

impl<E: EntityTrait> HasMany<E> {
    pub fn get(&self) -> &[E::Model] {
        &self.items
    }

    pub fn set(&mut self, items: Vec<E::Model>) {
        self.items = items
    }
}

macro_rules! impl_partial_eq_eq {
    ($ty:ident) => {
        impl<E: EntityTrait> PartialEq for $ty<E> {
            fn eq(&self, _: &Self) -> bool {
                // same type regard as true
                true
            }
        }

        impl<E: EntityTrait> Eq for $ty<E> {}
    };
}

impl_partial_eq_eq!(HasOne);
impl_partial_eq_eq!(HasMany);
