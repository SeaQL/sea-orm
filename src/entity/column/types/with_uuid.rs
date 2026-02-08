use super::*;
use crate::prelude::{TextUuid, Uuid};

impl<E: EntityTrait> UuidColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<Uuid>);
    bind_oper!(pub ne, ne, trait Into<Uuid>);
    bind_oper!(pub gt, gt, trait Into<Uuid>);
    bind_oper!(pub gte, gte, trait Into<Uuid>);
    bind_oper!(pub lt, lt, trait Into<Uuid>);
    bind_oper!(pub lte, lte, trait Into<Uuid>);

    bind_oper_2!(pub between, between, trait Into<Uuid>);
    bind_oper_2!(pub not_between, not_between, trait Into<Uuid>);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<Uuid>);

    bind_vec_func!(pub is_in, is_in, trait Into<Uuid>);
    bind_vec_func!(pub is_not_in, is_not_in, trait Into<Uuid>);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + Into<Uuid> + sea_query::postgres_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

impl<E: EntityTrait> TextUuidColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, type TextUuid);
    bind_oper!(pub ne, ne, type TextUuid);
    bind_oper!(pub gt, gt, type TextUuid);
    bind_oper!(pub gte, gte, type TextUuid);
    bind_oper!(pub lt, lt, type TextUuid);
    bind_oper!(pub lte, lte, type TextUuid);

    bind_oper_2!(pub between, between, type TextUuid);
    bind_oper_2!(pub not_between, not_between, type TextUuid);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, type TextUuid);

    bind_vec_func!(pub is_in, is_in, type TextUuid);
    bind_vec_func!(pub is_not_in, is_not_in, type TextUuid);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
