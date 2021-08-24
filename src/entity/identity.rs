use crate::{ColumnTrait, EntityTrait, IdenStatic};
use sea_query::{DynIden, IntoIden};

#[derive(Debug, Clone)]
pub enum Identity {
    Unary(DynIden),
    Binary(DynIden, DynIden),
    Ternary(DynIden, DynIden, DynIden),
}

pub trait IntoIdentity {
    fn into_identity(self) -> Identity;
}

pub trait IdentityOf<E>
where
    E: EntityTrait,
{
    fn identity_of(self) -> Identity;
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
