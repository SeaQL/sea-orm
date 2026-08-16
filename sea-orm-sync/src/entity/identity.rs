use crate::{ColumnTrait, EntityTrait, IdenStatic};
use sea_query::{Alias, DynIden, Iden, IntoIden, SeaRc};
use std::{borrow::Cow, fmt::Write};

/// A one-or-many column identifier — the abstraction SeaORM uses to refer
/// to either a single column or a composite (e.g. a composite primary key
/// or foreign key).
///
/// Specialized variants exist for the common arities (1/2/3 columns) to
/// avoid heap allocation; longer composites fall back to [`Identity::Many`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identity {
    /// A single column.
    Unary(DynIden),
    /// Two columns (composite).
    Binary(DynIden, DynIden),
    /// Three columns (composite).
    Ternary(DynIden, DynIden, DynIden),
    /// Four or more columns (composite).
    Many(Vec<DynIden>),
}

impl Identity {
    /// Number of columns in this identifier.
    pub fn arity(&self) -> usize {
        match self {
            Self::Unary(_) => 1,
            Self::Binary(_, _) => 2,
            Self::Ternary(_, _, _) => 3,
            Self::Many(vec) => vec.len(),
        }
    }

    /// Iterate over each column iden in order.
    pub fn iter(&self) -> BorrowedIdentityIter<'_> {
        BorrowedIdentityIter {
            identity: self,
            index: 0,
        }
    }

    /// `true` if `col` is one of the columns making up this identifier.
    pub fn contains(&self, col: &DynIden) -> bool {
        self.iter().any(|c| c == col)
    }

    /// `true` if every column of `other` also appears in `self`.
    pub fn fully_contains(&self, other: &Identity) -> bool {
        for col in other.iter() {
            if !self.contains(col) {
                return false;
            }
        }
        true
    }
}

impl IntoIterator for Identity {
    type Item = DynIden;
    type IntoIter = OwnedIdentityIter;

    fn into_iter(self) -> Self::IntoIter {
        OwnedIdentityIter {
            identity: self,
            index: 0,
        }
    }
}

impl Iden for Identity {
    fn quoted(&self) -> Cow<'static, str> {
        match self {
            Identity::Unary(iden) => iden.inner(),
            Identity::Binary(iden1, iden2) => Cow::Owned(format!("{iden1}{iden2}")),
            Identity::Ternary(iden1, iden2, iden3) => Cow::Owned(format!("{iden1}{iden2}{iden3}")),
            Identity::Many(vec) => {
                let mut s = String::new();
                for iden in vec.iter() {
                    write!(&mut s, "{iden}").expect("Infallible");
                }
                Cow::Owned(s)
            }
        }
    }

    fn to_string(&self) -> String {
        match self.quoted() {
            Cow::Borrowed(s) => s.to_owned(),
            Cow::Owned(s) => s,
        }
    }

    fn unquoted(&self) -> &str {
        panic!("Should not call this")
    }
}

/// Borrowing iterator over the columns of an [`Identity`].
#[derive(Debug)]
pub struct BorrowedIdentityIter<'a> {
    identity: &'a Identity,
    index: usize,
}

/// Owning iterator over the columns of an [`Identity`]
/// (returned by `IntoIterator for Identity`).
#[derive(Debug)]
pub struct OwnedIdentityIter {
    identity: Identity,
    index: usize,
}

impl<'a> Iterator for BorrowedIdentityIter<'a> {
    type Item = &'a DynIden;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.identity {
            Identity::Unary(iden1) => {
                if self.index == 0 {
                    Some(iden1)
                } else {
                    None
                }
            }
            Identity::Binary(iden1, iden2) => match self.index {
                0 => Some(iden1),
                1 => Some(iden2),
                _ => None,
            },
            Identity::Ternary(iden1, iden2, iden3) => match self.index {
                0 => Some(iden1),
                1 => Some(iden2),
                2 => Some(iden3),
                _ => None,
            },
            Identity::Many(vec) => vec.get(self.index),
        };
        self.index += 1;
        result
    }
}

impl Iterator for OwnedIdentityIter {
    type Item = DynIden;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match &self.identity {
            Identity::Unary(iden1) => {
                if self.index == 0 {
                    Some(iden1.clone())
                } else {
                    None
                }
            }
            Identity::Binary(iden1, iden2) => match self.index {
                0 => Some(iden1.clone()),
                1 => Some(iden2.clone()),
                _ => None,
            },
            Identity::Ternary(iden1, iden2, iden3) => match self.index {
                0 => Some(iden1.clone()),
                1 => Some(iden2.clone()),
                2 => Some(iden3.clone()),
                _ => None,
            },
            Identity::Many(vec) => vec.get(self.index).cloned(),
        };
        self.index += 1;
        result
    }
}

/// Conversion into an [`Identity`]. Implemented for `&str`/`String`, a single
/// column iden, and tuples of column idens (for composites up to 12 columns).
pub trait IntoIdentity {
    /// Build the [`Identity`].
    fn into_identity(self) -> Identity;
}

/// Conversion into an [`Identity`] whose columns are guaranteed to belong to
/// entity `E`. Used by [`RelationBuilder::from`](crate::RelationBuilder::from) /
/// [`to`](crate::RelationBuilder::to) so the type system enforces that
/// relation columns reference the right table.
pub trait IdentityOf<E>
where
    E: EntityTrait,
{
    /// Build the [`Identity`].
    fn identity_of(self) -> Identity;
}

impl IntoIdentity for Identity {
    fn into_identity(self) -> Identity {
        self
    }
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

macro_rules! impl_into_identity {
    ( $($T:ident : $N:tt),+ $(,)? ) => {
        impl< $($T),+ > IntoIdentity for ( $($T),+ )
        where
            $($T: IdenStatic),+
        {
            fn into_identity(self) -> Identity {
                Identity::Many(vec![
                    $(self.$N.into_iden()),+
                ])
            }
        }
    };
}

#[rustfmt::skip]
mod impl_into_identity {
    use super::*;

    impl_into_identity!(T0:0, T1:1, T2:2, T3:3);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8, T9:9);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8, T9:9, T10:10);
    impl_into_identity!(T0:0, T1:1, T2:2, T3:3, T4:4, T5:5, T6:6, T7:7, T8:8, T9:9, T10:10, T11:11);
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

macro_rules! impl_identity_of {
    ( $($T:ident),+ $(,)? ) => {
        impl<E, C> IdentityOf<E> for ( $($T),+ )
        where
            E: EntityTrait<Column = C>,
            C: ColumnTrait,
        {
            fn identity_of(self) -> Identity {
                self.into_identity()
            }
        }
    };
}

#[rustfmt::skip]
mod impl_identity_of {
    use super::*;

    impl_identity_of!(C, C);
    impl_identity_of!(C, C, C);
    impl_identity_of!(C, C, C, C);
    impl_identity_of!(C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C, C, C, C, C, C);
    impl_identity_of!(C, C, C, C, C, C, C, C, C, C, C, C);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_identity_contains() {
        let abc = Identity::Ternary("a".into(), "b".into(), "c".into());
        let a = Identity::Unary("a".into());
        let ab = Identity::Binary("a".into(), "b".into());
        let bc = Identity::Binary("b".into(), "c".into());
        let d = Identity::Unary("d".into());
        let bcd = Identity::Ternary("b".into(), "c".into(), "d".into());

        assert!(abc.contains(&"a".into()));
        assert!(abc.contains(&"b".into()));
        assert!(abc.contains(&"c".into()));
        assert!(!abc.contains(&"d".into()));

        assert!(abc.fully_contains(&a));
        assert!(abc.fully_contains(&ab));
        assert!(abc.fully_contains(&bc));
        assert!(!abc.fully_contains(&d));
        assert!(!abc.fully_contains(&bcd));
    }
}
