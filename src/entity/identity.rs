use crate::{ColumnTrait, EntityTrait};

// The original `sea_orm::Identity` enum and `sea_orm::IntoIdentity` trait were dropped since 0.11.0
// It was replaced by `sea_query::Identity` and `sea_query::IntoIdentity`, we reexport it here to keep the symbols
pub use sea_query::{Identity, IntoIdentity};

/// Check the [Identity] of an Entity
pub trait IdentityOf<E>
where
    E: EntityTrait,
{
    /// Method to call to perform this check
    fn identity_of(self) -> Identity;
}

impl<E, C> IdentityOf<E> for C
where
    E: EntityTrait<Column = C>,
    C: ColumnTrait,
{
    fn identity_of(self) -> Identity {
        self.into_identity()
    }
}

impl<E, C> IdentityOf<E> for (C, C)
where
    E: EntityTrait<Column = C>,
    C: ColumnTrait,
{
    fn identity_of(self) -> Identity {
        self.into_identity()
    }
}

impl<E, C> IdentityOf<E> for (C, C, C)
where
    E: EntityTrait<Column = C>,
    C: ColumnTrait,
{
    fn identity_of(self) -> Identity {
        self.into_identity()
    }
}
