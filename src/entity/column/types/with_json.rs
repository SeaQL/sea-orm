use super::*;
use crate::prelude::Json;

impl<E: EntityTrait> JsonColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<Json>);
    bind_oper!(pub ne, ne, trait Into<Json>);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<Json>);

    bind_vec_func!(pub is_in, is_in, trait Into<Json>);
    bind_vec_func!(pub is_not_in, is_not_in, trait Into<Json>);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V>(&self, v: impl IntoIterator<Item = V>) -> Expr
    where
        V: Into<Json>,
    {
        use itertools::Itertools;

        let vec = v.into_iter().map(Into::into).collect_vec();
        self.0.eq_any(vec)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
