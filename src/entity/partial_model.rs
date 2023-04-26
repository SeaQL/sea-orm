use crate::{FromQueryResult, SelectColumns};

/// A trait for a part of [Model](super::model::ModelTrait)
pub trait PartialModelTrait: FromQueryResult {
    /// Select specific columns this [PartialModel] needs
    fn select_cols<S: SelectColumns>(select: S) -> S;
}
