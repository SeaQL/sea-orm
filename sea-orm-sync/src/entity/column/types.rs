#![allow(missing_docs)]

use crate::{ColumnDef, ColumnTrait, DynIden, EntityTrait, ExprTrait, Iden, IntoSimpleExpr, Value};
use sea_query::{Expr, NumericValue, NumericValueNullable, SelectStatement};
use std::borrow::Cow;

pub trait IntoOption<T> {
    #[allow(dead_code)]
    fn into_option(self) -> Option<T>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoolColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(BoolColumn);

/// A column of numeric type, including integer, float and decimal
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NumericColumn<E: EntityTrait>(pub E::Column);
/// A column of numeric type, including integer, float and decimal that is also nullable
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NumericColumnNullable<E: EntityTrait>(pub E::Column);
impl_expr_traits!(NumericColumn);
impl_expr_traits!(NumericColumnNullable);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringColumn<E: EntityTrait>(pub E::Column);
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringColumnNullable<E: EntityTrait>(pub E::Column);
impl_expr_traits!(StringColumn);
impl_expr_traits!(StringColumnNullable);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BytesColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(BytesColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct JsonColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(JsonColumn);
// we dont need JsonColumnNullable because None can be converted to Json

/// Supports both chrono and time Date
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DateLikeColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(DateLikeColumn);

/// Supports both chrono and time Time
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TimeLikeColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(TimeLikeColumn);

/// Supports both chrono and time DateTime
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DateTimeLikeColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(DateTimeLikeColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct UuidColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(UuidColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TextUuidColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(TextUuidColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IpNetworkColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(IpNetworkColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NumericArrayColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(NumericArrayColumn);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GenericArrayColumn<E: EntityTrait>(pub E::Column);
impl_expr_traits!(GenericArrayColumn);

#[cfg(feature = "with-json")]
mod with_json;

#[cfg(any(feature = "with-chrono", feature = "with-time"))]
mod with_datetime;

#[cfg(feature = "with-uuid")]
mod with_uuid;

#[cfg(feature = "with-ipnetwork")]
mod with_ipnetwork;

#[cfg(feature = "postgres-array")]
mod postgres_array;

mod macros {
    macro_rules! impl_expr_traits {
        ($ty:ident) => {
            impl<E: EntityTrait> Iden for $ty<E> {
                fn quoted(&self) -> std::borrow::Cow<'static, str> {
                    self.0.quoted()
                }
                fn unquoted(&self) -> &str {
                    self.0.unquoted()
                }
            }

            impl<E: EntityTrait> IntoSimpleExpr for $ty<E> {
                fn into_simple_expr(self) -> Expr {
                    self.0.into_simple_expr()
                }
            }
        };
    }

    macro_rules! bind_oper_0 {
        ($vis:vis $op:ident, $bind_op:ident) => {
            $vis fn $op(&self) -> Expr {
                self.0.$bind_op()
            }
        };
    }

    macro_rules! bind_oper {
        ($vis:vis $op:ident, $bind_op:ident, trait $value_ty:path) => {
            $vis fn $op<V>(&self, v: V) -> Expr
            where
                V: Into<Value> + $value_ty,
            {
                self.0.$bind_op(v)
            }
        };
        ($vis:vis $op:ident, $bind_op:ident, type $value_ty:path) => {
            $vis fn $op<V>(&self, v: V) -> Expr
            where
                V: Into<$value_ty>,
            {
                self.0.$bind_op(v.into())
            }
        };
    }

    macro_rules! bind_expr_oper {
        ($vis:vis $op:ident, $bind_op:ident) => {
            $vis fn $op<T>(&self, expr: T) -> Expr
            where
                T: Into<Expr>,
            {
                Expr::col(self.as_column_ref()).$bind_op(expr)
            }
        };
    }

    macro_rules! bind_oper_2 {
        ($vis:vis $op:ident, $bind_op:ident, trait $value_ty:path) => {
            $vis fn $op<V>(&self, v1: V, v2: V) -> Expr
            where
                V: Into<Value> + $value_ty,
            {
                self.0.$bind_op(v1, v2)
            }
        };
        ($vis:vis $op:ident, $bind_op:ident, type $value_ty:path) => {
            $vis fn $op<V>(&self, v1: V, v2: V) -> Expr
            where
                V: Into<$value_ty>,
            {
                self.0.$bind_op(v1.into(), v2.into())
            }
        };
    }

    macro_rules! bind_vec_func {
        ($vis:vis $op:ident, $bind_op:ident, trait $value_ty:path) => {
            #[allow(clippy::wrong_self_convention)]
            $vis fn $op<V, I>(&self, v: I) -> Expr
            where
                V: Into<Value> + $value_ty,
                I: IntoIterator<Item = V>,
            {
                self.0.$bind_op(v)
            }
        };
        ($vis:vis $op:ident, $bind_op:ident, type $value_ty:path) => {
            #[allow(clippy::wrong_self_convention)]
            $vis fn $op<V, I>(&self, v: I) -> Expr
            where
                V: Into<$value_ty>,
                I: IntoIterator<Item = V>,
            {
                self.0.$bind_op(v.into_iter().map(|v| v.into()))
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

    pub(super) use bind_expr_oper;
    pub(super) use bind_oper;
    pub(super) use bind_oper_0;
    pub(super) use bind_oper_2;
    pub(super) use bind_subquery_func;
    pub(super) use bind_vec_func;
    pub(super) use boilerplate;
    pub(super) use impl_expr_traits;
}

use macros::*;

impl<E: EntityTrait> BoolColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<bool>);
    bind_oper!(pub ne, ne, trait Into<bool>);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<bool>);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}

macro_rules! impl_numeric_column {
    ($ty:ident, $trait:ident) => {
    impl<E: EntityTrait> $ty<E> {
        boilerplate!(pub);

        bind_oper!(pub eq, eq, trait $trait);
        bind_oper!(pub ne, ne, trait $trait);
        bind_oper!(pub gt, gt, trait NumericValue);
        bind_oper!(pub gte, gte, trait NumericValue);
        bind_oper!(pub lt, lt, trait NumericValue);
        bind_oper!(pub lte, lte, trait NumericValue);

        bind_expr_oper!(pub add, add);
        bind_expr_oper!(pub sub, sub);
        bind_expr_oper!(pub div, div);
        bind_expr_oper!(pub mul, mul);

        bind_oper_2!(pub between, between, trait NumericValue);
        bind_oper_2!(pub not_between, not_between, trait NumericValue);

        bind_oper_0!(pub max, max);
        bind_oper_0!(pub min, min);
        bind_oper_0!(pub sum, sum);
        bind_oper_0!(pub count, count);
        bind_oper_0!(pub is_null, is_null);
        bind_oper_0!(pub is_not_null, is_not_null);

        bind_oper!(pub if_null, if_null, trait $trait);

        bind_vec_func!(pub is_in, is_in, trait $trait);
        bind_vec_func!(pub is_not_in, is_not_in, trait $trait);

        /// `= ANY(..)` operator. Postgres only.
        #[cfg(feature = "postgres-array")]
        pub fn eq_any<V, I>(&self, v: I) -> Expr
        where
            V: Into<Value> + $trait + sea_query::ValueType + sea_query::with_array::NotU8,
            I: IntoIterator<Item = V>,
        {
            self.0.eq_any(v)
        }

        bind_subquery_func!(pub in_subquery);
        bind_subquery_func!(pub not_in_subquery);
    }
};}
impl_numeric_column!(NumericColumn, NumericValue);
impl_numeric_column!(NumericColumnNullable, NumericValueNullable);

macro_rules! impl_string_column {
    ($ty:ident, $trait:path) => {
    impl<E: EntityTrait> $ty<E> {
        boilerplate!(pub);

        bind_oper!(pub eq, eq, trait $trait);
        bind_oper!(pub ne, ne, trait $trait);
        bind_oper!(pub gt, gt, trait Into<String>);
        bind_oper!(pub gte, gte, trait Into<String>);
        bind_oper!(pub lt, lt, trait Into<String>);
        bind_oper!(pub lte, lte, trait Into<String>);

        bind_oper_2!(pub between, between, trait Into<String>);
        bind_oper_2!(pub not_between, not_between, trait Into<String>);

        bind_oper!(pub like, like, trait Into<String>);
        bind_oper!(pub not_like, not_like, trait Into<String>);
        bind_oper!(pub ilike, ilike, trait Into<String>);
        bind_oper!(pub not_ilike, not_ilike, trait Into<String>);
        bind_oper!(pub starts_with, starts_with, trait Into<String>);
        bind_oper!(pub ends_with, ends_with, trait Into<String>);
        bind_oper!(pub contains, contains, trait Into<String>);

        bind_oper_0!(pub count, count);
        bind_oper_0!(pub is_null, is_null);
        bind_oper_0!(pub is_not_null, is_not_null);

        bind_oper!(pub if_null, if_null, trait $trait);

        bind_vec_func!(pub is_in, is_in, trait $trait);
        bind_vec_func!(pub is_not_in, is_not_in, trait $trait);

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
};}

impl IntoOption<String> for Cow<'_, str> {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}
impl IntoOption<String> for &'_ str {
    fn into_option(self) -> Option<String> {
        Some(self.into())
    }
}
impl IntoOption<String> for String {
    fn into_option(self) -> Option<String> {
        Some(self)
    }
}
impl IntoOption<String> for Option<&'_ str> {
    fn into_option(self) -> Option<String> {
        self.map(Into::into)
    }
}
impl IntoOption<String> for Option<Cow<'_, str>> {
    fn into_option(self) -> Option<String> {
        self.map(Into::into)
    }
}
impl IntoOption<String> for Option<String> {
    fn into_option(self) -> Option<String> {
        self
    }
}

impl_string_column!(StringColumn, Into<String>);
impl_string_column!(StringColumnNullable, IntoOption<String>);

impl<E: EntityTrait> BytesColumn<E> {
    boilerplate!(pub);

    bind_oper!(pub eq, eq, trait Into<Vec<u8>>);
    bind_oper!(pub ne, ne, trait Into<Vec<u8>>);

    bind_oper_0!(pub count, count);
    bind_oper_0!(pub is_null, is_null);
    bind_oper_0!(pub is_not_null, is_not_null);

    bind_oper!(pub if_null, if_null, trait Into<Vec<u8>>);

    bind_subquery_func!(pub in_subquery);
    bind_subquery_func!(pub not_in_subquery);
}
