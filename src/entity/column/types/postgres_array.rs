use super::*;
use crate::ExprTrait;
use sea_query::extension::postgres::{PgBinOper, PgExpr};

macro_rules! bind_array_oper {
    ($vis:vis $op:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + sea_query::ValueType + sea_query::with_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            let vec: Vec<_> = v.into_iter().collect();
            Expr::col(self.as_column_ref()).$op(self.0.save_as(Expr::val(vec)))
        }
    };
    ($vis:vis $op:ident, trait $value_ty:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + $value_ty + sea_query::ValueType + sea_query::with_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            let vec: Vec<_> = v.into_iter().collect();
            Expr::col(self.as_column_ref()).$op(vec)
        }
    };
    ($vis:vis $op:ident, PgBinOper $oper:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + sea_query::ValueType + sea_query::with_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            let vec: Vec<_> = v.into_iter().collect();
            Expr::col(self.as_column_ref()).binary(PgBinOper::$oper, self.0.save_as(Expr::val(vec)))
        }
    };
    ($vis:vis $op:ident, PgBinOper $oper:ident, trait $value_ty:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + $value_ty + sea_query::ValueType + sea_query::with_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            let vec: Vec<_> = v.into_iter().collect();
            Expr::col(self.as_column_ref()).binary(PgBinOper::$oper, vec)
        }
    };
}

impl<E: EntityTrait> NumericArrayColumn<E> {
    boilerplate!(pub);

    bind_array_oper!(pub eq, trait NumericValue);
    bind_array_oper!(pub ne, trait NumericValue);
    bind_array_oper!(pub contains, trait NumericValue);
    bind_array_oper!(pub contained, trait NumericValue);
    bind_array_oper!(pub overlap, PgBinOper Overlap, trait NumericValue);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

impl<E: EntityTrait> GenericArrayColumn<E> {
    boilerplate!(pub);

    bind_array_oper!(pub eq);
    bind_array_oper!(pub ne);
    bind_array_oper!(pub contains);
    bind_array_oper!(pub contained);
    bind_array_oper!(pub overlap, PgBinOper Overlap);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
