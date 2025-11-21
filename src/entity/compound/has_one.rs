use crate::{EntityTrait, HasOneModel};
use std::hash::{Hash, Hasher};

#[derive(Debug, Default, Clone)]
pub enum HasOne<E: EntityTrait> {
    #[default]
    Unloaded,
    NotFound,
    Loaded(Box<E::ModelEx>),
}

impl<E: EntityTrait> HasOne<E> {
    /// Construct a `HasOne::Loaded` value
    pub fn loaded<M: Into<E::ModelEx>>(model: M) -> Self {
        Self::Loaded(Box::new(model.into()))
    }

    /// Return true if variant is `Unloaded`
    pub fn is_unloaded(&self) -> bool {
        matches!(self, HasOne::Unloaded)
    }

    /// Return true if variant is `NotFound`
    pub fn is_not_found(&self) -> bool {
        matches!(self, HasOne::NotFound)
    }

    /// Return true if variant is `Loaded`
    pub fn is_loaded(&self) -> bool {
        matches!(self, HasOne::Loaded(_))
    }

    /// True if variant is `Unloaded` or `NotFound`
    pub fn is_none(&self) -> bool {
        matches!(self, HasOne::Unloaded | HasOne::NotFound)
    }

    /// Get a reference, if loaded
    pub fn as_ref(&self) -> Option<&E::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(model.as_ref()),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }

    /// Get a mutable reference, if loaded
    pub fn as_mut(&mut self) -> Option<&mut E::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(model),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }

    /// Convert into an `Option<ModelEx>`
    pub fn into_option(self) -> Option<E::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(*model),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }

    /// Take ownership of the contained Model, leaving `Unloaded` in place.
    pub fn take(&mut self) -> Option<E::ModelEx> {
        std::mem::take(self).into_option()
    }

    /// # Panics
    ///
    /// Panics if called on `Unloaded` or `NotFound` values.
    pub fn unwrap(self) -> E::ModelEx {
        match self {
            HasOne::Loaded(model) => *model,
            HasOne::Unloaded => panic!("called `HasOne::unwrap()` on an `Unloaded` value"),
            HasOne::NotFound => panic!("called `HasOne::unwrap()` on a `NotFound` value"),
        }
    }
}

impl<E> HasOne<E>
where
    E: EntityTrait,
    E::ActiveModelEx: From<E::ModelEx>,
{
    pub fn into_active_model(self) -> HasOneModel<E> {
        match self {
            HasOne::Loaded(_) => {
                let model = self.unwrap();
                let active_model: E::ActiveModelEx = model.into();
                HasOneModel::Set(active_model.into())
            }
            HasOne::Unloaded => HasOneModel::NotSet,
            HasOne::NotFound => HasOneModel::NotSet,
        }
    }
}

impl<E> PartialEq for HasOne<E>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
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
    E::ModelEx: Eq,
{
}

// Option<Box<ModelEx<E>>> <-> HasOne<E> conversions and comparisons
impl<E: EntityTrait> From<HasOne<E>> for Option<Box<E::ModelEx>> {
    fn from(value: HasOne<E>) -> Self {
        match value {
            HasOne::Loaded(model) => Some(model),
            HasOne::Unloaded | HasOne::NotFound => None,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<E::ModelEx>>> for HasOne<E> {
    fn from(value: Option<Box<E::ModelEx>>) -> Self {
        match value {
            Some(model) => HasOne::Loaded(model),
            None => HasOne::NotFound,
        }
    }
}

impl<E> PartialEq<Option<Box<E::ModelEx>>> for HasOne<E>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &Option<Box<E::ModelEx>>) -> bool {
        match (self, other) {
            (HasOne::Loaded(a), Some(b)) => a.as_ref() == b.as_ref(),
            (HasOne::Unloaded | HasOne::NotFound, None) => true,
            _ => false,
        }
    }
}

impl<E> PartialEq<HasOne<E>> for Option<Box<E::ModelEx>>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
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
        match <Option<E::ModelEx>>::deserialize(deserializer)? {
            Some(model) => Ok(HasOne::Loaded(Box::new(model))),
            None => Ok(HasOne::Unloaded),
        }
    }
}
