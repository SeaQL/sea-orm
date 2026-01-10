use super::*;
use crate::prelude::IpNetwork;

impl<E: EntityTrait> IpNetworkColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<IpNetwork>);
    bind_oper!(pub ne, ne, trait Into<IpNetwork>);
    bind_oper!(pub gt, gt, trait Into<IpNetwork>);
    bind_oper!(pub gte, gte, trait Into<IpNetwork>);
    bind_oper!(pub lt, lt, trait Into<IpNetwork>);
    bind_oper!(pub lte, lte, trait Into<IpNetwork>);

    bind_oper_2!(pub between, between, trait Into<IpNetwork>);
    bind_oper_2!(pub not_between, not_between, trait Into<IpNetwork>);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<IpNetwork>);

    bind_vec_func!(pub is_in, is_in, trait Into<IpNetwork>);
    bind_vec_func!(pub is_not_in, is_not_in, trait Into<IpNetwork>);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + Into<IpNetwork> + sea_query::ValueType + sea_query::with_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
