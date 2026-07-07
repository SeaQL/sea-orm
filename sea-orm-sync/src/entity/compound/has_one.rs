use crate::{ActiveHasOne, EntityTrait};
use std::hash::{Hash, Hasher};

#[doc(hidden)]
pub trait HasOneCardinality {
    type Entity: EntityTrait;
    type ModelEx;
    type ActiveModelEx;
}

impl<E> HasOneCardinality for E
where
    E: EntityTrait,
{
    type Entity = E;
    type ModelEx = Box<E::ModelEx>;
    type ActiveModelEx = Box<E::ActiveModelEx>;
}

impl<E> HasOneCardinality for Option<E>
where
    E: EntityTrait,
{
    type Entity = E;
    type ModelEx = Option<Box<E::ModelEx>>;
    type ActiveModelEx = Option<Box<E::ActiveModelEx>>;
}

#[derive_where::derive_where(Debug, Clone; <T as HasOneCardinality>::ModelEx)]
#[derive(Default)]
pub enum HasOne<T>
where
    T: HasOneCardinality,
{
    #[default]
    Unloaded,
    Loaded(<T as HasOneCardinality>::ModelEx),
}

impl<T> HasOne<T>
where
    T: HasOneCardinality,
{
    /// Construct a `HasOne::Loaded` value. Accepts a bare model for a required
    /// relation, or an `Option<model>` for an optional one.
    pub fn loaded(model: impl IntoHasOneLoaded<T>) -> Self {
        model.into_has_one_loaded()
    }

    /// Return true if variant is `Unloaded`
    pub fn is_unloaded(&self) -> bool {
        matches!(self, HasOne::Unloaded)
    }

    /// Return true if variant is `Loaded`
    pub fn is_loaded(&self) -> bool {
        matches!(self, HasOne::Loaded(_))
    }
}

/// Conversion used by [`HasOne::loaded`]. Wraps a bare model (required relation)
/// or an `Option<model>` (optional relation) into the correct `Loaded` payload,
/// selected by the relation's cardinality type parameter.
#[doc(hidden)]
pub trait IntoHasOneLoaded<T: HasOneCardinality> {
    fn into_has_one_loaded(self) -> HasOne<T>;
}

impl<E, M> IntoHasOneLoaded<E> for M
where
    E: EntityTrait,
    M: Into<E::ModelEx>,
{
    fn into_has_one_loaded(self) -> HasOne<E> {
        HasOne::Loaded(Box::new(self.into()))
    }
}

impl<E, M> IntoHasOneLoaded<Option<E>> for Option<M>
where
    E: EntityTrait,
    M: Into<E::ModelEx>,
{
    fn into_has_one_loaded(self) -> HasOne<Option<E>> {
        HasOne::Loaded(self.map(|model| Box::new(model.into())))
    }
}

impl<E> HasOne<E>
where
    E: EntityTrait,
{
    /// Required relations only have no value when they are unloaded.
    pub fn is_none(&self) -> bool {
        matches!(self, HasOne::Unloaded)
    }

    /// Get a reference, if loaded
    pub fn as_ref(&self) -> Option<&E::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(model.as_ref()),
            HasOne::Unloaded => None,
        }
    }

    /// Get a mutable reference, if loaded
    pub fn as_mut(&mut self) -> Option<&mut E::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(model),
            HasOne::Unloaded => None,
        }
    }

    /// Convert into an `Option<ModelEx>`
    pub fn into_option(self) -> Option<E::ModelEx> {
        match self {
            HasOne::Loaded(model) => Some(*model),
            HasOne::Unloaded => None,
        }
    }

    /// Take ownership of the contained Model, leaving `Unloaded` in place.
    pub fn take(&mut self) -> Option<E::ModelEx> {
        std::mem::take(self).into_option()
    }

    /// # Panics
    ///
    /// Panics if called on an `Unloaded` value.
    pub fn unwrap(self) -> E::ModelEx {
        match self {
            HasOne::Loaded(model) => *model,
            HasOne::Unloaded => panic!("called `HasOne::unwrap()` on an `Unloaded` value"),
        }
    }
}

impl<E> HasOne<Option<E>>
where
    E: EntityTrait,
{
    /// Return true if this optional relation was loaded and no model was found.
    pub fn is_not_found(&self) -> bool {
        matches!(self, HasOne::Loaded(None))
    }

    /// True if variant is `Unloaded` or `Loaded(None)`
    pub fn is_none(&self) -> bool {
        matches!(self, HasOne::Unloaded | HasOne::Loaded(None))
    }

    /// Get a reference, if loaded with a model
    pub fn as_ref(&self) -> Option<&E::ModelEx> {
        match self {
            HasOne::Loaded(Some(model)) => Some(model.as_ref()),
            HasOne::Unloaded | HasOne::Loaded(None) => None,
        }
    }

    /// Get a mutable reference, if loaded with a model
    pub fn as_mut(&mut self) -> Option<&mut E::ModelEx> {
        match self {
            HasOne::Loaded(Some(model)) => Some(model),
            HasOne::Unloaded | HasOne::Loaded(None) => None,
        }
    }

    /// Convert into an `Option<ModelEx>`
    pub fn into_option(self) -> Option<E::ModelEx> {
        match self {
            HasOne::Loaded(Some(model)) => Some(*model),
            HasOne::Unloaded | HasOne::Loaded(None) => None,
        }
    }

    /// Take ownership of the contained Model, leaving `Unloaded` in place.
    pub fn take(&mut self) -> Option<E::ModelEx> {
        std::mem::take(self).into_option()
    }

    /// # Panics
    ///
    /// Panics if called on an `Unloaded` or `Loaded(None)` value.
    pub fn unwrap(self) -> E::ModelEx {
        match self {
            HasOne::Loaded(Some(model)) => *model,
            HasOne::Unloaded => panic!("called `HasOne::unwrap()` on an `Unloaded` value"),
            HasOne::Loaded(None) => panic!("called `HasOne::unwrap()` on a `Loaded(None)` value"),
        }
    }
}

impl<E> HasOne<E>
where
    E: EntityTrait,
    E::ActiveModelEx: From<E::ModelEx>,
{
    pub fn into_active_model(self) -> ActiveHasOne<E> {
        match self {
            HasOne::Loaded(model) => ActiveHasOne::<E>::set(*model),
            HasOne::Unloaded => ActiveHasOne::NotSet,
        }
    }
}

impl<E> HasOne<Option<E>>
where
    E: EntityTrait,
    E::ActiveModelEx: From<E::ModelEx>,
{
    pub fn into_active_model(self) -> ActiveHasOne<Option<E>> {
        match self {
            HasOne::Loaded(Some(model)) => ActiveHasOne::<Option<E>>::set(Some(*model)),
            HasOne::Unloaded | HasOne::Loaded(None) => ActiveHasOne::NotSet,
        }
    }
}

impl<T> PartialEq for HasOne<T>
where
    T: HasOneCardinality,
    <T as HasOneCardinality>::ModelEx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HasOne::Unloaded, HasOne::Unloaded) => true,
            (HasOne::Loaded(a), HasOne::Loaded(b)) => a == b,
            _ => false,
        }
    }
}

impl<T> Eq for HasOne<T>
where
    T: HasOneCardinality,
    <T as HasOneCardinality>::ModelEx: Eq,
{
}

impl<E: EntityTrait> From<HasOne<E>> for Option<Box<E::ModelEx>> {
    fn from(value: HasOne<E>) -> Self {
        match value {
            HasOne::Loaded(model) => Some(model),
            HasOne::Unloaded => None,
        }
    }
}

impl<E: EntityTrait> From<HasOne<Option<E>>> for Option<Box<E::ModelEx>> {
    fn from(value: HasOne<Option<E>>) -> Self {
        match value {
            HasOne::Loaded(model) => model,
            HasOne::Unloaded => None,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<E::ModelEx>>> for HasOne<E> {
    fn from(value: Option<Box<E::ModelEx>>) -> Self {
        match value {
            Some(model) => HasOne::Loaded(model),
            None => HasOne::Unloaded,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<E::ModelEx>>> for HasOne<Option<E>> {
    fn from(value: Option<Box<E::ModelEx>>) -> Self {
        HasOne::Loaded(value)
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
            (HasOne::Unloaded, None) => true,
            _ => false,
        }
    }
}

impl<E> PartialEq<Option<Box<E::ModelEx>>> for HasOne<Option<E>>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &Option<Box<E::ModelEx>>) -> bool {
        match (self, other) {
            (HasOne::Loaded(a), b) => a == b,
            (HasOne::Unloaded, None) => true,
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

impl<E> PartialEq<HasOne<Option<E>>> for Option<Box<E::ModelEx>>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &HasOne<Option<E>>) -> bool {
        other == self
    }
}

impl<T> Hash for HasOne<T>
where
    T: HasOneCardinality,
    <T as HasOneCardinality>::ModelEx: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Loaded(model) => model.hash(state),
            Self::Unloaded => {}
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
            HasOne::Loaded(model) => Some(model),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "with-json")]
impl<E> serde::Serialize for HasOne<Option<E>>
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
            HasOne::Loaded(model) => model.as_ref(),
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

#[cfg(feature = "with-json")]
impl<'de, E> serde::Deserialize<'de> for HasOne<Option<E>>
where
    E: EntityTrait,
    E::ModelEx: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<E::ModelEx>>::deserialize(deserializer)? {
            Some(model) => Ok(HasOne::Loaded(Some(Box::new(model)))),
            None => Ok(HasOne::Unloaded),
        }
    }
}
