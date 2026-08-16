use crate::{EntityTrait, FromQueryResult, IdenStatic, Iterable, ModelTrait, QuerySelect};
use sea_query::Expr;

/// A partial projection of a [`Model`](super::model::ModelTrait) — a struct
/// that holds only some of the entity's columns, used to avoid overfetching
/// in `SELECT` queries.
///
/// Derive it on a custom struct with `#[derive(DerivePartialModel)]` and
/// `#[sea_orm(entity = "...")]`, then call
/// [`Select::into_partial_model`](crate::Select::into_partial_model) to
/// restrict the query to just those columns. Nested partial models can be
/// composed with `#[sea_orm(nested)]` fields.
pub trait PartialModelTrait: FromQueryResult {
    /// Add the partial model's columns to a [`QuerySelect`]'s projection.
    ///
    /// No need to implement this method; implement `select_cols_nested` instead.
    fn select_cols<S: QuerySelect>(select: S) -> S {
        Self::select_cols_nested(select, None, None)
    }

    /// Used when nesting these structs into each other.
    ///
    /// Example impl
    ///
    /// ```ignore
    /// fn select_cols_nested<S: QuerySelect>(mut select: S, prefix: Option<&str>) -> S {
    ///     if let Some(prefix) = prefix {
    ///         for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
    ///             let alias = format!("{prefix}{}", col.as_str());
    ///             select = select.column_as(col, alias);
    ///         }
    ///     } else {
    ///         for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
    ///             select = select.column(col);
    ///         }
    ///     }
    ///     select
    /// }
    /// ```
    fn select_cols_nested<S: QuerySelect>(
        select: S,
        prefix: Option<&str>,
        alias: Option<&'static str>,
    ) -> S;
}

impl<T: PartialModelTrait> PartialModelTrait for Option<T> {
    fn select_cols_nested<S: QuerySelect>(
        select: S,
        prefix: Option<&str>,
        alias: Option<&'static str>,
    ) -> S {
        T::select_cols_nested(select, prefix, alias)
    }
}

impl<T: ModelTrait + FromQueryResult> PartialModelTrait for T {
    fn select_cols_nested<S: QuerySelect>(
        mut select: S,
        prefix: Option<&str>,
        alias: Option<&'static str>,
    ) -> S {
        match (prefix, alias) {
            (Some(prefix), Some(alias)) => {
                for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
                    let select_as = format!("{prefix}{}", col.as_str());
                    select = select.column_as(Expr::col((alias, col)), select_as);
                }
            }
            (Some(prefix), None) => {
                for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
                    let select_as = format!("{prefix}{}", col.as_str());
                    select = select.column_as(col, select_as);
                }
            }
            (None, Some(alias)) => {
                for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
                    select = select.column_as(Expr::col((alias, col)), col.as_str());
                }
            }
            (None, None) => {
                for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
                    select = select.column(col);
                }
            }
        }
        select
    }
}
