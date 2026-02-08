use super::*;
use sea_query::{DateLikeValue, DateTimeLikeValue, TimeLikeValue};

impl<E: EntityTrait> DateLikeColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait DateLikeValue);
    bind_oper!(pub ne, ne, trait DateLikeValue);
    bind_oper!(pub gt, gt, trait DateLikeValue);
    bind_oper!(pub gte, gte, trait DateLikeValue);
    bind_oper!(pub lt, lt, trait DateLikeValue);
    bind_oper!(pub lte, lte, trait DateLikeValue);

    bind_oper_2!(pub between, between, trait DateLikeValue);
    bind_oper_2!(pub not_between, not_between, trait DateLikeValue);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait DateLikeValue);

    bind_vec_func!(pub is_in, is_in, trait DateLikeValue);
    bind_vec_func!(pub is_not_in, is_not_in, trait DateLikeValue);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + DateLikeValue + sea_query::ValueType + sea_query::postgres_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

impl<E: EntityTrait> TimeLikeColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait TimeLikeValue);
    bind_oper!(pub ne, ne, trait TimeLikeValue);
    bind_oper!(pub gt, gt, trait TimeLikeValue);
    bind_oper!(pub gte, gte, trait TimeLikeValue);
    bind_oper!(pub lt, lt, trait TimeLikeValue);
    bind_oper!(pub lte, lte, trait TimeLikeValue);

    bind_oper_2!(pub between, between, trait TimeLikeValue);
    bind_oper_2!(pub not_between, not_between, trait TimeLikeValue);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait TimeLikeValue);

    bind_vec_func!(pub is_in, is_in, trait TimeLikeValue);
    bind_vec_func!(pub is_not_in, is_not_in, trait TimeLikeValue);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + TimeLikeValue + sea_query::ValueType + sea_query::postgres_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

impl<E: EntityTrait> DateTimeLikeColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait DateTimeLikeValue);
    bind_oper!(pub ne, ne, trait DateTimeLikeValue);
    bind_oper!(pub gt, gt, trait DateTimeLikeValue);
    bind_oper!(pub gte, gte, trait DateTimeLikeValue);
    bind_oper!(pub lt, lt, trait DateTimeLikeValue);
    bind_oper!(pub lte, lte, trait DateTimeLikeValue);

    bind_oper_2!(pub between, between, trait DateTimeLikeValue);
    bind_oper_2!(pub not_between, not_between, trait DateTimeLikeValue);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait DateTimeLikeValue);

    bind_vec_func!(pub is_in, is_in, trait DateTimeLikeValue);
    bind_vec_func!(pub is_not_in, is_not_in, trait DateTimeLikeValue);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + DateTimeLikeValue + sea_query::postgres_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
