#![allow(missing_docs)]

use crate::{ColumnDef, ColumnTrait, DynIden, EntityTrait, IntoSimpleExpr, Value};
use sea_query::{Expr, NumericValue, SelectStatement};

macro_rules! impl_into_simple_expr {
    ($ty:ident) => {
        impl<E: EntityTrait> IntoSimpleExpr for $ty<E> {
            fn into_simple_expr(self) -> Expr {
                self.0.into_simple_expr()
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoolColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(BoolColumn);

/// A column of numeric type, including integer, float and decimal
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NumericColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(NumericColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(StringColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CharColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(CharColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BytesColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(BytesColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct JsonColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(JsonColumn);

/// Supports both chrono and time Date
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DateLikeColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(DateLikeColumn);

/// Supports both chrono and time Time
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TimeLikeColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(TimeLikeColumn);

/// Supports both chrono and time DateTime
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DateTimeLikeColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(DateTimeLikeColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct UuidColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(UuidColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VectorColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(VectorColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IpNetworkColumn<E: EntityTrait>(pub E::Column);
impl_into_simple_expr!(IpNetworkColumn);

#[cfg(feature = "with-json")]
mod with_json;

#[cfg(feature = "with-chrono")]
mod with_chrono;

#[cfg(feature = "with-time")]
mod with_time;

#[cfg(feature = "with-rust_decimal")]
mod with_rust_decimal;

#[cfg(feature = "with-bigdecimal")]
mod with_bigdecimal;

#[cfg(feature = "with-uuid")]
mod with_uuid;

#[cfg(feature = "with-ipnetwork")]
mod with_ipnetwork;

mod macros {
    macro_rules! bind_oper_0 {
        ($vis:vis $op:ident, $bind_op:ident) => {
            $vis fn $op(&self) -> Expr {
                self.0.$bind_op()
            }
        };
    }

    macro_rules! bind_oper {
        ($vis:vis $op:ident, $bind_op:ident, trait $value_ty:ident) => {
            $vis fn $op<V>(&self, v: V) -> Expr
            where
                V: Into<Value> + $value_ty,
            {
                self.0.$bind_op(v)
            }
        };
        ($vis:vis $op:ident, $bind_op:ident, type $value_ty:ident) => {
            $vis fn $op<V>(&self, v: V) -> Expr
            where
                V: Into<Value> + Into<$value_ty>,
            {
                self.0.$bind_op(v)
            }
        };
    }

    macro_rules! bind_oper_2 {
        ($vis:vis $op:ident, $bind_op:ident, trait $value_ty:ident) => {
            $vis fn $op<V>(&self, v1: V, v2: V) -> Expr
            where
                V: Into<Value> + $value_ty,
            {
                self.0.$bind_op(v1, v2)
            }
        };
        ($vis:vis $op:ident, $bind_op:ident, type $value_ty:ident) => {
            $vis fn $op<V>(&self, v1: V, v2: V) -> Expr
            where
                V: Into<Value> + Into<$value_ty>,
            {
                self.0.$bind_op(v1, v2)
            }
        };
    }

    macro_rules! bind_vec_func {
        ($vis:vis $op:ident, $bind_op:ident, trait $value_ty:ident) => {
            #[allow(clippy::wrong_self_convention)]
            $vis fn $op<V, I>(&self, v: I) -> Expr
            where
                V: Into<Value> + $value_ty,
                I: IntoIterator<Item = V>,
            {
                self.0.$bind_op(v)
            }
        };
        ($vis:vis $op:ident, $bind_op:ident, type $value_ty:ident) => {
            #[allow(clippy::wrong_self_convention)]
            $vis fn $op<V, I>(&self, v: I) -> Expr
            where
                V: Into<Value> + Into<$value_ty>,
                I: IntoIterator<Item = V>,
            {
                self.0.$bind_op(v)
            }
        };
    }

    macro_rules! bind_subquery_func {
        ($vis:vis $func:ident) => {
            #[allow(clippy::wrong_self_convention)]
            $vis fn $func(&self, s: SelectStatement) -> Expr {
                self.0.$func(s)
            }
        };
    }

    macro_rules! boilerplate {
        ($vis:vis) => {
            /// Get the column definition with SQL attributes
            $vis fn def(&self) -> ColumnDef {
                self.0.def()
            }

            /// Get the enum type name if this is a enum column
            $vis fn enum_type_name(&self) -> Option<&'static str> {
                self.0.enum_type_name()
            }

            /// Get the name of the entity the column belongs to
            $vis fn entity_name(&self) -> DynIden {
                self.0.entity_name()
            }

            /// Get the table.column reference
            $vis fn as_column_ref(&self) -> (DynIden, DynIden) {
                self.0.as_column_ref()
            }
        };
    }

    pub(super) use bind_oper;
    pub(super) use bind_oper_0;
    pub(super) use bind_oper_2;
    pub(super) use bind_subquery_func;
    pub(super) use bind_vec_func;
    pub(super) use boilerplate;
}

use macros::*;

impl<E: EntityTrait> NumericColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait NumericValue);
    bind_oper!(pub ne, ne, trait NumericValue);
    bind_oper!(pub gt, gt, trait NumericValue);
    bind_oper!(pub gte, gte, trait NumericValue);
    bind_oper!(pub lt, lt, trait NumericValue);
    bind_oper!(pub lte, lte, trait NumericValue);

    bind_oper_2!(pub between, between, trait NumericValue);
    bind_oper_2!(pub not_between, not_between, trait NumericValue);

    bind_oper_0!(pub max, max);
    bind_oper_0!(pub min, min);
    bind_oper_0!(pub sum, sum);
    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait NumericValue);

    bind_vec_func!(pub is_in, is_in, trait NumericValue);
    bind_vec_func!(pub is_not_in, is_not_in, trait NumericValue);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + NumericValue + sea_query::ValueType + sea_query::with_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

impl<E: EntityTrait> StringColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, type String);
    bind_oper!(pub ne, ne, type String);
    bind_oper!(pub gt, gt, type String);
    bind_oper!(pub gte, gte, type String);
    bind_oper!(pub lt, lt, type String);
    bind_oper!(pub lte, lte, type String);

    bind_oper_2!(pub between, between, type String);
    bind_oper_2!(pub not_between, not_between, type String);

    bind_oper!(pub like, like, type String);
    bind_oper!(pub not_like, not_like, type String);
    bind_oper!(pub ilike, ilike, type String);
    bind_oper!(pub not_ilike, not_ilike, type String);
    bind_oper!(pub starts_with, starts_with, type String);
    bind_oper!(pub ends_with, ends_with, type String);
    bind_oper!(pub contains, contains, type String);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, type String);

    bind_vec_func!(pub is_in, is_in, type String);
    bind_vec_func!(pub is_not_in, is_not_in, type String);

    /// `= ANY(..)` operator. Postgres only.
    #[cfg(feature = "postgres-array")]
    pub fn eq_any<V, I>(&self, v: I) -> Expr
    where
        V: Into<Value> + Into<String> + sea_query::ValueType + sea_query::with_array::NotU8,
        I: IntoIterator<Item = V>,
    {
        self.0.eq_any(v)
    }

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
