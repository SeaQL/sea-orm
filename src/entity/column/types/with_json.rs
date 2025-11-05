use super::*;
use crate::prelude::Json;

impl<E: EntityTrait> JsonColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, type Json);
    bind_oper!(pub ne, ne, type Json);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, type Json);

    bind_vec_func!(pub is_in, is_in, type Json);
    bind_vec_func!(pub is_not_in, is_not_in, type Json);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + Into<Json> + sea_query::ValueType + sea_query::with_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
