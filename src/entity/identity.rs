use sea_query::{Iden, IntoIden};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Identity {
    Unary(Rc<dyn Iden>),
    // Binary((Rc<dyn Iden>, Rc<dyn Iden>)),
    // Ternary((Rc<dyn Iden>, Rc<dyn Iden>, Rc<dyn Iden>)),
}

pub trait IntoIdentity {
    fn into_identity(self) -> Identity;
}

impl Identity {
    pub fn into_iden(self) -> Rc<dyn Iden> {
        match self {
            Self::Unary(iden) => iden,
        }
    }
}

impl<T> IntoIdentity for T
where
    T: IntoIden,
{
    fn into_identity(self) -> Identity {
        Identity::Unary(self.into_iden())
    }
}
