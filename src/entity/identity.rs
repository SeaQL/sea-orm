use crate::IdenStatic;
use sea_query::{DynIden, IntoIden};

#[derive(Debug, Clone)]
pub enum Identity {
    Unary(DynIden),
    Binary(DynIden, DynIden),
    // Ternary(DynIden, DynIden, DynIden),
}

pub trait IntoIdentity {
    fn into_identity(self) -> Identity;
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
