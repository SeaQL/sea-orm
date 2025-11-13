use crate::EntityTrait;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default, Clone)]
pub enum HasOne<E: EntityTrait> {
    #[default]
    Unloaded,
    NotFound,
    Loaded(Box<<E as EntityTrait>::ModelEx>),
}

impl<E: EntityTrait> HasOne<E> {
    /// Construct a `HasOne::Loaded` value
    pub fn loaded<M: Into<<E as EntityTrait>::ModelEx>>(model: M) -> Self {
        Self::Loaded(Box::new(model.into()))
    }

    pub fn is_unloaded(&self) -> bool {
        matches!(self, HasOne::Unloaded)
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, HasOne::NotFound)
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, HasOne::Loaded(_))
    }

    /// True if variant is `Unloaded` or `NotFound`
    pub fn is_none(&self) -> bool {
        matches!(self, HasOne::Unloaded | HasOne::NotFound)
    }

    pub fn as_ref(&self) -> Option<&<E as EntityTrait>::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(model.as_ref()),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut <E as EntityTrait>::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(model),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }

    pub fn into_option(self) -> Option<<E as EntityTrait>::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(*model),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }

    /// # Panics
    ///
    /// Panics if called on `Unloaded` or `NotFound` values.
    pub fn unwrap(self) -> <E as EntityTrait>::ModelEx {
        match self {
            HasOne::Loaded(model) => *model,
            HasOne::Unloaded => panic!("called `HasOne::unwrap()` on an `Unloaded` value"),
            HasOne::NotFound => panic!("called `HasOne::unwrap()` on a `NotFound` value"),
        }
    }
}

impl<E> PartialEq for HasOne<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HasOne::Unloaded, HasOne::Unloaded) => true,
            (HasOne::NotFound, HasOne::NotFound) => true,
            (HasOne::Loaded(a), HasOne::Loaded(b)) => a == b,
            _ => false,
        }
    }
}

impl<E> Eq for HasOne<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: Eq,
{
}

// Option<Box<ModelEx<E>>> <-> HasOne<E> conversions and comparisons
impl<E: EntityTrait> From<HasOne<E>> for Option<Box<<E as EntityTrait>::ModelEx>> {
    fn from(value: HasOne<E>) -> Self {
        match value {
            HasOne::Loaded(model) => Some(model),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<<E as EntityTrait>::ModelEx>>> for HasOne<E> {
    fn from(value: Option<Box<<E as EntityTrait>::ModelEx>>) -> Self {
        match value {
            Some(model) => HasOne::Loaded(model),
            None => HasOne::NotFound,
        }
    }
}

impl<E> PartialEq<Option<Box<<E as EntityTrait>::ModelEx>>> for HasOne<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq,
{
    fn eq(&self, other: &Option<Box<<E as EntityTrait>::ModelEx>>) -> bool {
        match (self, other) {
            (HasOne::Loaded(a), Some(b)) => a.as_ref() == b.as_ref(),
            (HasOne::Unloaded | HasOne::NotFound, None) => true,
            _ => false,
        }
    }
}

impl<E> PartialEq<HasOne<E>> for Option<Box<<E as EntityTrait>::ModelEx>>
where
    E: EntityTrait,
    <E as EntityTrait>::ModelEx: PartialEq,
{
    fn eq(&self, other: &HasOne<E>) -> bool {
        other == self
    }
}

impl<E> Hash for HasOne<E>
where
    E: EntityTrait,
    E::ModelEx: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Loaded(model) => model.hash(state),
            Self::Unloaded => {}
            Self::NotFound => {}
        }
    }
}

#[cfg(feature = "with-json")]
impl<E> serde::Serialize for HasOne<E>
where
    E: EntityTrait,
    E::ModelEx: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            HasOne::Unloaded => None,
            HasOne::NotFound => None,
            HasOne::Loaded(model) => Some(model),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "with-json")]
impl<'de, E> serde::Deserialize<'de> for HasOne<E>
where
    E: EntityTrait,
    E::ModelEx: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<<E as EntityTrait>::ModelEx>>::deserialize(deserializer)? {
            Some(model) => Ok(HasOne::Loaded(Box::new(model))),
            None => Ok(HasOne::Unloaded),
        }
    }
}
