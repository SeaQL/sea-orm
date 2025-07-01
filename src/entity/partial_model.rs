use crate::{EntityTrait, FromQueryResult, IdenStatic, Iterable, ModelTrait, SelectColumns};

/// A trait for a part of [Model](super::model::ModelTrait)
pub trait PartialModelTrait: FromQueryResult {
    /// Select specific columns this [PartialModel] needs
    ///
    /// No need to implement this method, please implement `select_cols_nested` instead.
    fn select_cols<S: SelectColumns>(select: S) -> S {
        Self::select_cols_nested(select, None)
    }

    /// Used when nesting these structs into each other.
    ///
    /// Example impl
    ///
    /// ```ignore
    /// fn select_cols_nested<S: SelectColumns>(mut select: S, prefix: Option<&str>) -> S {
    ///     if let Some(prefix) = prefix {
    ///         for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
    ///             let alias = format!("{prefix}{}", col.as_str());
    ///             select = select.select_column_as(col, alias);
    ///         }
    ///     } else {
    ///         for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
    ///             select = select.select_column(col);
    ///         }
    ///     }
    ///     select
    /// }
    /// ```
    fn select_cols_nested<S: SelectColumns>(select: S, _prefix: Option<&str>) -> S;
}

impl<T: PartialModelTrait> PartialModelTrait for Option<T> {
    fn select_cols_nested<S: SelectColumns>(select: S, prefix: Option<&str>) -> S {
        T::select_cols_nested(select, prefix)
    }
}

impl<T: ModelTrait + FromQueryResult> PartialModelTrait for T {
    fn select_cols_nested<S: SelectColumns>(mut select: S, prefix: Option<&str>) -> S {
        if let Some(prefix) = prefix {
            for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
                let alias = format!("{prefix}{}", col.as_str());
                select = select.select_column_as(col, alias);
            }
        } else {
            for col in <<T::Entity as EntityTrait>::Column as Iterable>::iter() {
                select = select.select_column(col);
            }
        }
        select
    }
}
