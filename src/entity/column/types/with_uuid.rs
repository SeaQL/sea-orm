use super::*;
use crate::prelude::Uuid;

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
    pub fn eq_any<V>(&self, v: impl IntoIterator<Item = V>) -> Expr
    where
        V: Into<Uuid>,
    {
        use itertools::Itertools;

        let vec = v.into_iter().map(Into::into).collect_vec();
        self.0.eq_any(vec)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
