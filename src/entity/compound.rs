#![allow(missing_docs)]
use super::{ColumnTrait, EntityTrait, PrimaryKeyToColumn, PrimaryKeyTrait};
use crate::{IntoSimpleExpr, Iterable, QueryFilter, QueryOrder, Related};
use sea_query::{IntoValueTuple, Order, TableRef};
mod has_many;
mod has_one;

pub use has_many::{HasMany, Iter as HasManyIter};
pub use has_one::HasOne;

pub trait EntityLoaderTrait<E: EntityTrait>: QueryFilter + QueryOrder {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadTarget {
    TableRef(TableRef),
    Relation(String),
}

pub trait EntityLoaderWithParam<E: EntityTrait> {
    fn into_with_param(self) -> (LoadTarget, Option<LoadTarget>);
}

impl<E, R> EntityLoaderWithParam<E> for R
where
    E: EntityTrait,
    R: EntityTrait,
    E: Related<R>,
{
    fn into_with_param(self) -> (LoadTarget, Option<LoadTarget>) {
        (LoadTarget::TableRef(self.table_ref()), None)
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
    fn into_with_param(self) -> (LoadTarget, Option<LoadTarget>) {
        (
            LoadTarget::TableRef(self.0.table_ref()),
            Some(LoadTarget::TableRef(self.1.table_ref())),
        )
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
