use core::ops::Index;
use std::slice;

use super::super::EntityTrait;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "E::ModelEx: serde::Serialize",
        deserialize = "E::ModelEx: serde::Deserialize<'de>"
    ))
)]
pub enum HasMany<E: EntityTrait> {
    Unloaded,
    Loaded(Vec<<E as EntityTrait>::ModelEx>),
}

impl<E> PartialEq for HasMany<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HasMany::Unloaded, HasMany::Unloaded) => true,
            (HasMany::Loaded(a), HasMany::Loaded(b)) => a == b,
            _ => false,
        }
    }
}

impl<E> Eq for HasMany<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: Eq,
{
}

impl<E: EntityTrait> Default for HasMany<E> {
    fn default() -> Self {
        Self::Unloaded
    }
}

impl<E: EntityTrait> HasMany<E> {
    pub fn is_unloaded(&self) -> bool {
        matches!(self, HasMany::Unloaded)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            HasMany::Unloaded => true,
            HasMany::Loaded(models) => models.is_empty(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&<E as EntityTrait>::ModelEx> {
        match self {
            HasMany::Loaded(models) => models.get(index),
            HasMany::Unloaded => None,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            HasMany::Loaded(models) => models.len(),
            HasMany::Unloaded => 0,
        }
    }
}

impl<E: EntityTrait> From<HasMany<E>> for Option<Vec<<E as EntityTrait>::ModelEx>> {
    fn from(value: HasMany<E>) -> Self {
        match value {
            HasMany::Loaded(models) => Some(models),
            HasMany::Unloaded => None,
        }
    }
}

impl<E: EntityTrait> Index<usize> for HasMany<E> {
    type Output = <E as EntityTrait>::ModelEx;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            HasMany::Unloaded => {
                panic!("index out of bounds: the HasMany is Unloaded (index: {index})")
            }
            HasMany::Loaded(items) => items.index(index),
        }
    }
}

impl<E: EntityTrait> IntoIterator for HasMany<E> {
    type Item = <E as EntityTrait>::ModelEx;
    type IntoIter = std::vec::IntoIter<<E as EntityTrait>::ModelEx>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            HasMany::Loaded(models) => models.into_iter(),
            HasMany::Unloaded => Vec::new().into_iter(),
        }
    }
}

impl<E: EntityTrait> From<Vec<<E as EntityTrait>::ModelEx>> for HasMany<E> {
    fn from(value: Vec<<E as EntityTrait>::ModelEx>) -> Self {
        HasMany::Loaded(value)
    }
}

impl<E, const N: usize> PartialEq<[<E as EntityTrait>::Model; N]> for HasMany<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq<<E as EntityTrait>::Model>,
    <E as EntityTrait>::Model: PartialEq<<E as EntityTrait>::ModelEx>,
{
    fn eq(&self, other: &[<E as EntityTrait>::Model; N]) -> bool {
        match self {
            HasMany::Loaded(models) => models.as_slice() == other.as_slice(),
            HasMany::Unloaded => false,
        }
    }
}

impl<E, const N: usize> PartialEq<HasMany<E>> for [<E as EntityTrait>::Model; N]
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq<<E as EntityTrait>::Model>,
    <E as EntityTrait>::Model: PartialEq<<E as EntityTrait>::ModelEx>,
{
    fn eq(&self, other: &HasMany<E>) -> bool {
        other == self
    }
}

impl<E> PartialEq<[<E as EntityTrait>::Model]> for HasMany<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq<<E as EntityTrait>::Model>,
    <E as EntityTrait>::Model: PartialEq<<E as EntityTrait>::ModelEx>,
{
    fn eq(&self, other: &[<E as EntityTrait>::Model]) -> bool {
        match self {
            HasMany::Loaded(models) => models.as_slice() == other,
            HasMany::Unloaded => false,
        }
    }
}

impl<E> PartialEq<HasMany<E>> for [<E as EntityTrait>::Model]
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq<<E as EntityTrait>::Model>,
    <E as EntityTrait>::Model: PartialEq<<E as EntityTrait>::ModelEx>,
{
    fn eq(&self, other: &HasMany<E>) -> bool {
        other == self
    }
}

#[derive(Debug, Clone)]
pub struct HasManyIter<'a, E: EntityTrait> {
    pub(crate) inner: Option<slice::Iter<'a, <E as EntityTrait>::ModelEx>>,
}

impl<E: EntityTrait> HasMany<E> {
    pub fn iter(&self) -> HasManyIter<'_, E> {
        HasManyIter {
            inner: match self {
                HasMany::Loaded(models) => Some(models.iter()),
                HasMany::Unloaded => None,
            },
        }
    }
}

impl<'a, E: EntityTrait> Iterator for HasManyIter<'a, E> {
    type Item = &'a <E as EntityTrait>::ModelEx;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut().and_then(|iter| iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner
            .as_ref()
            .map(|iter| iter.size_hint())
            .unwrap_or((0, Some(0)))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.map(|iter| iter.count()).unwrap_or(0)
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.inner.and_then(|iter| iter.last())
    }
}

impl<'a, E: EntityTrait> ExactSizeIterator for HasManyIter<'a, E> {
    fn len(&self) -> usize {
        self.inner.as_ref().map(|iter| iter.len()).unwrap_or(0)
    }
}

impl<'a, E: EntityTrait> DoubleEndedIterator for HasManyIter<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.as_mut().and_then(|iter| iter.next_back())
    }
}

impl<'a, E: EntityTrait> IntoIterator for &'a HasMany<E> {
    type Item = &'a <E as EntityTrait>::ModelEx;
    type IntoIter = HasManyIter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        HasManyIter {
            inner: match self {
                HasMany::Loaded(models) => Some(models.iter()),
                HasMany::Unloaded => None,
            },
        }
    }
}
