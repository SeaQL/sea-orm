use crate::{ColumnTrait, EntityTrait, IdenStatic};
use sea_query::{Alias, DynIden, Iden, IntoIden, SeaRc};
use std::fmt;

/// Defines an operation for an Entity
#[derive(Debug, Clone)]
pub enum Identity {
    /// Performs one operation
    Unary(DynIden),
    /// Performs two operations
    Binary(DynIden, DynIden),
    /// Performs three operations
    Ternary(DynIden, DynIden, DynIden),
}

impl Iden for Identity {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        match self {
            Identity::Unary(iden) => {
                write!(s, "{}", iden.to_string()).unwrap();
            }
            Identity::Binary(iden1, iden2) => {
                write!(s, "{}", iden1.to_string()).unwrap();
                write!(s, "{}", iden2.to_string()).unwrap();
            }
            Identity::Ternary(iden1, iden2, iden3) => {
                write!(s, "{}", iden1.to_string()).unwrap();
                write!(s, "{}", iden2.to_string()).unwrap();
                write!(s, "{}", iden3.to_string()).unwrap();
            }
        }
    }
}

/// Performs a conversion into an [Identity]
pub trait IntoIdentity {
    /// Method to perform the conversion
    fn into_identity(self) -> Identity;
}

/// Check the [Identity] of an Entity
pub trait IdentityOf<E>
where
    E: EntityTrait,
{
    /// Method to call to perform this check
    fn identity_of(self) -> Identity;
}

impl IntoIdentity for String {
    fn into_identity(self) -> Identity {
        self.as_str().into_identity()
    }
}

impl IntoIdentity for &str {
    fn into_identity(self) -> Identity {
        Identity::Unary(SeaRc::new(Alias::new(self)))
    }
}

impl<T> IntoIdentity for T
where
    T: IdenStatic,
{
    fn into_identity(self) -> Identity {
        Identity::Unary(self.into_iden())
    }
}

impl<T, C> IntoIdentity for (T, C)
where
    T: IdenStatic,
    C: IdenStatic,
{
    fn into_identity(self) -> Identity {
        Identity::Binary(self.0.into_iden(), self.1.into_iden())
    }
}

impl<T, C, R> IntoIdentity for (T, C, R)
where
    T: IdenStatic,
    C: IdenStatic,
    R: IdenStatic,
{
    fn into_identity(self) -> Identity {
        Identity::Ternary(self.0.into_iden(), self.1.into_iden(), self.2.into_iden())
    }
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
