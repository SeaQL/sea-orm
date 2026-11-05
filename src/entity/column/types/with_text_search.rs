use super::*;
use crate::prelude::{TsQuery, TsVector};

impl<E: EntityTrait> TsVectorColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<TsVector>);
    bind_oper!(pub ne, ne, trait Into<TsVector>);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<TsVector>);

    bind_vec_func!(pub is_in, is_in, trait Into<TsVector>);
    bind_vec_func!(pub is_not_in, is_not_in, trait Into<TsVector>);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);

    /// Match operator (`@@`). Postgres only.
    pub fn matches<V>(&self, query: V) -> Expr
    where
        V: Into<Expr>,
    {
        use sea_query::extension::postgres::PgBinOper;
        Expr::col(self.0.as_column_ref()).binary(PgBinOper::Matches, query.into())
    }
}

impl<E: EntityTrait> TsQueryColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<TsQuery>);
    bind_oper!(pub ne, ne, trait Into<TsQuery>);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<TsQuery>);

    bind_vec_func!(pub is_in, is_in, trait Into<TsQuery>);
    bind_vec_func!(pub is_not_in, is_not_in, trait Into<TsQuery>);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);

    /// Match operator (`@@`). Postgres only.
    pub fn matches<V>(&self, vector: V) -> Expr
    where
        V: Into<Expr>,
    {
        use sea_query::extension::postgres::PgBinOper;
        Expr::col(self.0.as_column_ref()).binary(PgBinOper::Matches, vector.into())
    }
}
