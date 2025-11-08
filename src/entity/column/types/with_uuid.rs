use super::*;
use crate::prelude::Uuid;

impl<E: EntityTrait> UuidColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, type Uuid);
    bind_oper!(pub ne, ne, type Uuid);
    bind_oper!(pub gt, gt, type Uuid);
    bind_oper!(pub gte, gte, type Uuid);
    bind_oper!(pub lt, lt, type Uuid);
    bind_oper!(pub lte, lte, type Uuid);

    bind_oper_2!(pub between, between, type Uuid);
    bind_oper_2!(pub not_between, not_between, type Uuid);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, type Uuid);

    bind_vec_func!(pub is_in, is_in, type Uuid);
    bind_vec_func!(pub is_not_in, is_not_in, type Uuid);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + Into<Uuid> + sea_query::ValueType + sea_query::with_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
