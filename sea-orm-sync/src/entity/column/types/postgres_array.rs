use super::*;
use crate::ExprTrait;

macro_rules! bind_array_oper {
    ($vis:vis $op:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + sea_query::ValueType + sea_query::postgres_array::NotU8,
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
            V: Into<Value> + $value_ty + sea_query::ValueType + sea_query::postgres_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            let vec: Vec<_> = v.into_iter().collect();
            Expr::col(self.as_column_ref()).$op(self.0.save_as(Expr::val(vec)))
        }
    };
    ($vis:vis $op:ident, $func:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + sea_query::ValueType + sea_query::postgres_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            self.0.$func(v)
        }
    };
    ($vis:vis $op:ident, $func:ident, trait $value_ty:ident) => {
        /// Postgres only.
        $vis fn $op<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + $value_ty + sea_query::ValueType + sea_query::postgres_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            self.0.$func(v)
        }
    };
}

impl<E: EntityTrait> NumericArrayColumn<E> {
    boilerplate!(pub);

    bind_array_oper!(pub eq, trait NumericValue);
    bind_array_oper!(pub ne, trait NumericValue);
    bind_array_oper!(pub contains, array_contains, trait NumericValue);
    bind_array_oper!(pub contained, array_contained, trait NumericValue);
    bind_array_oper!(pub overlap, array_overlap, trait NumericValue);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

impl<E: EntityTrait> GenericArrayColumn<E> {
    boilerplate!(pub);

    bind_array_oper!(pub eq);
    bind_array_oper!(pub ne);
    bind_array_oper!(pub contains, array_contains);
    bind_array_oper!(pub contained, array_contained);
    bind_array_oper!(pub overlap, array_overlap);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
