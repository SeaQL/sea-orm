#![allow(missing_docs)]
use super::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait};
use crate::{Iterable, QueryFilter};
use sea_query::IntoValueTuple;

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
    pub(crate) item: Option<Box<E::Model>>,
}

pub type BelongsTo<E> = HasOne<E>;

// TODO impl serde

impl<E: EntityTrait> Default for HasOne<E> {
    fn default() -> Self {
        Self { item: None }
    }
}

impl<E: EntityTrait> HasOne<E> {
    pub fn new<T: Into<Option<E::Model>>>(item: T) -> Self {
        Self {
            item: item.into().map(Into::into),
        }
    }

    pub fn get(&self) -> Option<&E::Model> {
        self.item.as_deref()
    }

    pub fn set<T: Into<Box<E::Model>>>(&mut self, item: Option<T>) {
        self.item = item.map(Into::into);
    }

    pub fn take(&mut self) -> Option<Box<E::Model>> {
        self.item.take()
    }
}

#[derive(Debug, Clone)]
pub struct HasMany<E: EntityTrait> {
    pub(crate) items: Vec<E::Model>,
}

impl<E: EntityTrait> Default for HasMany<E> {
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<E: EntityTrait> HasMany<E> {
    pub fn new(items: Vec<E::Model>) -> Self {
        Self { items }
    }

    pub fn get(&self) -> &[E::Model] {
        &self.items
    }

    pub fn set(&mut self, items: Vec<E::Model>) {
        self.items = items
    }

    pub fn take(&mut self) -> Vec<E::Model> {
        std::mem::take(&mut self.items)
    }
}

macro_rules! impl_partial_eq_eq {
    ($ty:ident, $field:ident) => {
        impl<E> PartialEq for $ty<E>
        where
            E: EntityTrait,
            E::Model: PartialEq,
        {
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }

        impl<E> Eq for $ty<E>
        where
            E: EntityTrait,
            E::Model: Eq,
        {
        }
    };
}

impl_partial_eq_eq!(HasOne, item);
impl_partial_eq_eq!(HasMany, items);

macro_rules! impl_serde {
    ($ty:ident, $field:ident, $field_type:ty) => {
        impl<E> serde::Serialize for $ty<E>
        where
            E: EntityTrait,
            E::Model: serde::Serialize,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.$field.serialize(serializer)
            }
        }

        impl<'de, E> serde::Deserialize<'de> for $ty<E>
        where
            E: EntityTrait,
            E::Model: serde::Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                <$field_type>::deserialize(deserializer).map($ty::new)
            }
        }
    };
}

#[cfg(feature = "with-json")]
impl_serde!(HasOne, item, Option<E::Model>);

#[cfg(feature = "with-json")]
impl_serde!(HasMany, items, Vec<E::Model>);
