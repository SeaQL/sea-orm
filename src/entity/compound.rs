#![allow(missing_docs)]
use super::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait};
use crate::{Iterable, QueryFilter, Related};
use sea_query::{IntoValueTuple, TableRef};

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

pub trait EntityLoaderWithParam<E: EntityTrait> {
    fn into_with_param(self) -> (TableRef, Option<TableRef>);
}

#[derive(Debug, Clone)]
pub struct HasOne<E: EntityTrait> {
    pub(crate) item: Option<Box<E::ModelEx>>,
}

pub type BelongsTo<E> = HasOne<E>;

// TODO impl serde

impl<E: EntityTrait> Default for HasOne<E> {
    fn default() -> Self {
        Self { item: None }
    }
}

impl<E: EntityTrait> HasOne<E> {
    pub fn new<T: Into<E::ModelEx>>(item: Option<T>) -> Self {
        Self {
            item: item.map(Into::into).map(Into::into),
        }
    }

    pub fn get(&self) -> Option<&E::ModelEx> {
        self.item.as_deref()
    }

    pub fn set<T: Into<E::ModelEx>>(&mut self, item: Option<T>) {
        self.item = item.map(Into::into).map(Into::into);
    }

    pub fn take(&mut self) -> Option<Box<E::ModelEx>> {
        self.item.take()
    }
}

#[derive(Debug, Clone)]
pub struct HasMany<E: EntityTrait> {
    pub(crate) items: Vec<E::ModelEx>,
}

impl<E: EntityTrait> Default for HasMany<E> {
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<E: EntityTrait> HasMany<E> {
    pub fn new(items: Vec<E::ModelEx>) -> Self {
        Self { items }
    }

    pub fn new_item(items: Vec<E::ModelEx>) -> Self {
        Self { items }
    }

    pub fn get(&self) -> &[E::ModelEx] {
        &self.items
    }

    pub fn set(&mut self, items: Vec<E::ModelEx>) {
        self.items = items
    }

    pub fn take(&mut self) -> Vec<E::ModelEx> {
        std::mem::take(&mut self.items)
    }
}

impl<E, R> EntityLoaderWithParam<E> for R
where
    E: EntityTrait,
    R: EntityTrait,
    E: Related<R>,
{
    fn into_with_param(self) -> (TableRef, Option<TableRef>) {
        (self.table_ref(), None)
    }
}

impl<E, R, S> EntityLoaderWithParam<E> for (R, S)
where
    E: EntityTrait,
    R: EntityTrait,
    E: Related<R>,
    S: EntityTrait,
    R: Related<S>,
{
    fn into_with_param(self) -> (TableRef, Option<TableRef>) {
        (self.0.table_ref(), Some(self.1.table_ref()))
    }
}

macro_rules! impl_partial_eq_eq {
    ($ty:ident, $field:ident) => {
        impl<E> PartialEq for $ty<E>
        where
            E: EntityTrait,
            E::ModelEx: PartialEq,
        {
            fn eq(&self, other: &Self) -> bool {
                self.$field == other.$field
            }
        }

        impl<E> Eq for $ty<E>
        where
            E: EntityTrait,
            E::ModelEx: Eq,
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
            E::ModelEx: serde::Serialize,
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
            E::ModelEx: serde::Deserialize<'de>,
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
impl_serde!(HasOne, item, Option<E::ModelEx>);

#[cfg(feature = "with-json")]
impl_serde!(HasMany, items, Vec<E::ModelEx>);

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
