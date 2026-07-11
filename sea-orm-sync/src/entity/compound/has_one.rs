use crate::{ActiveHasOne, EntityTrait};

#[derive_where::derive_where(Debug, Clone, PartialEq, Eq, Hash; E::ModelEx)]
#[derive(Default)]
pub enum HasOne<E>
where
    E: EntityTrait,
{
    #[default]
    Unloaded,
    Loaded(Option<Box<E::ModelEx>>),
}

impl<E> HasOne<E>
where
    E: EntityTrait,
{
    pub fn loaded(model: Option<impl Into<E::ModelEx>>) -> Self {
        Self::Loaded(model.map(|model| Box::new(model.into())))
    }

    /// Return true if variant is `Unloaded`
    pub fn is_unloaded(&self) -> bool {
        matches!(self, Self::Unloaded)
    }

    /// Return true if variant is `Loaded`
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }

    /// Return true if this relation was loaded and no model was found.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Loaded(None))
    }

    /// True if variant is `Unloaded` or `Loaded(None)`
    pub fn is_none(&self) -> bool {
        matches!(self, Self::Unloaded | Self::Loaded(None))
    }

    /// Get a reference, if loaded with a model
    pub fn as_ref(&self) -> Option<&E::ModelEx> {
        match self {
            Self::Loaded(Some(model)) => Some(model.as_ref()),
            Self::Unloaded | Self::Loaded(None) => None,
        }
    }

    /// Get a mutable reference, if loaded with a model
    pub fn as_mut(&mut self) -> Option<&mut E::ModelEx> {
        match self {
            Self::Loaded(Some(model)) => Some(model),
            Self::Unloaded | Self::Loaded(None) => None,
        }
    }

    /// Convert into an `Option<ModelEx>`
    pub fn into_option(self) -> Option<E::ModelEx> {
        match self {
            Self::Loaded(Some(model)) => Some(*model),
            Self::Unloaded | Self::Loaded(None) => None,
        }
    }

    /// # Panics
    ///
    /// Panics if called on an `Unloaded` or `Loaded(None)` value.
    pub fn unwrap(self) -> E::ModelEx {
        match self {
            Self::Loaded(Some(model)) => *model,
            Self::Unloaded => panic!("called `HasOne::unwrap()` on an `Unloaded` value"),
            Self::Loaded(None) => panic!("called `HasOne::unwrap()` on a `Loaded(None)` value"),
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
            Self::Loaded(Some(model)) => ActiveHasOne::set(Some(*model)),
            Self::Unloaded | Self::Loaded(None) => ActiveHasOne::NotSet,
        }
    }
}

impl<E: EntityTrait> From<HasOne<E>> for Option<Box<E::ModelEx>> {
    fn from(value: HasOne<E>) -> Self {
        match value {
            HasOne::Loaded(model) => model,
            HasOne::Unloaded => None,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<E::ModelEx>>> for HasOne<E> {
    fn from(value: Option<Box<E::ModelEx>>) -> Self {
        Self::Loaded(value)
    }
}

impl<E> PartialEq<Option<Box<E::ModelEx>>> for HasOne<E>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &Option<Box<E::ModelEx>>) -> bool {
        match (self, other) {
            (Self::Loaded(a), b) => a == b,
            (Self::Unloaded, None) => true,
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
            Some(model) => Ok(HasOne::Loaded(Some(Box::new(model)))),
            None => Ok(HasOne::Unloaded),
        }
    }
}
