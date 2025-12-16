use super::*;
use crate::ExprTrait;

macro_rules! bind_array_oper {
    ($vis:vis $op:ident) => {
        /// Postgres only.
        $vis fn $op<A>(&self, arr: A) -> Expr
        where
            A: Into<sea_query::value::Array>,
        {
            let value = Value::Array(arr.into());
            Expr::col(self.as_column_ref()).$op(self.0.save_as(Expr::val(value)))
        }
    };
    ($vis:vis $op:ident, trait $value_ty:ident) => {
        /// Postgres only.
        $vis fn $op<A>(&self, arr: A) -> Expr
        where
            A: Into<sea_query::value::Array>,
        {
            let value = Value::Array(arr.into());
            Expr::col(self.as_column_ref()).$op(self.0.save_as(Expr::val(value)))
        }
    };
    ($vis:vis $op:ident, $func:ident) => {
        /// Postgres only.
        $vis fn $op<A>(&self, arr: A) -> Expr
        where
            A: Into<sea_query::value::Array>,
        {
            self.0.$func(arr)
        }
    };
    ($vis:vis $op:ident, $func:ident, trait $value_ty:ident) => {
        /// Postgres only.
        $vis fn $op<A>(&self, arr: A) -> Expr
        where
            A: Into<sea_query::value::Array>,
        {
            self.0.$func(arr)
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
