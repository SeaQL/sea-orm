use crate::{ActiveBelongsTo, EntityTrait};

#[doc(hidden)]
pub trait BelongsToCardinality {
    type Entity: EntityTrait;
    type Loaded;
    type Set;
    type Value<M>;

    fn into_loaded<M>(value: Self::Value<M>) -> Self::Loaded
    where
        M: Into<<Self::Entity as EntityTrait>::ModelEx>;

    fn into_set<M>(value: Self::Value<M>) -> Self::Set
    where
        M: Into<<Self::Entity as EntityTrait>::ActiveModelEx>;
}

impl<E> BelongsToCardinality for E
where
    E: EntityTrait,
{
    type Entity = E;
    type Loaded = Box<E::ModelEx>;
    type Set = Box<E::ActiveModelEx>;
    type Value<M> = M;

    fn into_loaded<M>(value: M) -> Self::Loaded
    where
        M: Into<E::ModelEx>,
    {
        Box::new(value.into())
    }

    fn into_set<M>(value: M) -> Self::Set
    where
        M: Into<E::ActiveModelEx>,
    {
        Box::new(value.into())
    }
}

impl<E> BelongsToCardinality for Option<E>
where
    E: EntityTrait,
{
    type Entity = E;
    type Loaded = Option<Box<E::ModelEx>>;
    type Set = Option<Box<E::ActiveModelEx>>;
    type Value<M> = Option<M>;

    fn into_loaded<M>(value: Option<M>) -> Self::Loaded
    where
        M: Into<E::ModelEx>,
    {
        value.map(|model| Box::new(model.into()))
    }

    fn into_set<M>(value: Option<M>) -> Self::Set
    where
        M: Into<E::ActiveModelEx>,
    {
        value.map(|model| Box::new(model.into()))
    }
}

#[derive_where::derive_where(
    Debug, Clone, PartialEq, Eq, Hash;
    <T as BelongsToCardinality>::Loaded
)]
#[derive(Default)]
pub enum BelongsTo<T>
where
    T: BelongsToCardinality,
{
    #[default]
    Unloaded,
    Loaded(<T as BelongsToCardinality>::Loaded),
}

impl<T> BelongsTo<T>
where
    T: BelongsToCardinality,
{
    pub fn loaded<M>(model: T::Value<M>) -> Self
    where
        M: Into<<<T as BelongsToCardinality>::Entity as EntityTrait>::ModelEx>,
    {
        Self::Loaded(T::into_loaded(model))
    }

    /// Return true if variant is `Unloaded`
    pub fn is_unloaded(&self) -> bool {
        matches!(self, Self::Unloaded)
    }

    /// Return true if variant is `Loaded`
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }
}

impl<E> BelongsTo<E>
where
    E: EntityTrait,
{
    /// Get a reference, if loaded
    pub fn as_ref(&self) -> Option<&E::ModelEx> {
        match self {
            Self::Loaded(model) => Some(model.as_ref()),
            Self::Unloaded => None,
        }
    }

    /// Get a mutable reference, if loaded
    pub fn as_mut(&mut self) -> Option<&mut E::ModelEx> {
        match self {
            Self::Loaded(model) => Some(model),
            Self::Unloaded => None,
        }
    }

    /// Convert into an `Option<ModelEx>`
    pub fn into_option(self) -> Option<E::ModelEx> {
        match self {
            Self::Loaded(model) => Some(*model),
            Self::Unloaded => None,
        }
    }

    /// # Panics
    ///
    /// Panics if called on an `Unloaded` value.
    pub fn unwrap(self) -> E::ModelEx {
        match self {
            Self::Loaded(model) => *model,
            Self::Unloaded => panic!("called `BelongsTo::unwrap()` on an `Unloaded` value"),
        }
    }
}

impl<E> BelongsTo<Option<E>>
where
    E: EntityTrait,
{
    /// Return true if this optional relation was loaded and no model was found.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Loaded(None))
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
            Self::Unloaded => panic!("called `BelongsTo::unwrap()` on an `Unloaded` value"),
            Self::Loaded(None) => panic!("called `BelongsTo::unwrap()` on a `Loaded(None)` value"),
        }
    }
}

impl<E> BelongsTo<E>
where
    E: EntityTrait,
    E::ActiveModelEx: From<E::ModelEx>,
{
    pub fn into_active_model(self) -> ActiveBelongsTo<E> {
        match self {
            Self::Loaded(model) => ActiveBelongsTo::set(*model),
            Self::Unloaded => ActiveBelongsTo::NotSet,
        }
    }
}

impl<E> BelongsTo<Option<E>>
where
    E: EntityTrait,
    E::ActiveModelEx: From<E::ModelEx>,
{
    pub fn into_active_model(self) -> ActiveBelongsTo<Option<E>> {
        match self {
            Self::Loaded(Some(model)) => ActiveBelongsTo::set(Some(*model)),
            Self::Unloaded | Self::Loaded(None) => ActiveBelongsTo::NotSet,
        }
    }
}

impl<E: EntityTrait> From<BelongsTo<E>> for Option<Box<E::ModelEx>> {
    fn from(value: BelongsTo<E>) -> Self {
        match value {
            BelongsTo::Loaded(model) => Some(model),
            BelongsTo::Unloaded => None,
        }
    }
}

impl<E: EntityTrait> From<BelongsTo<Option<E>>> for Option<Box<E::ModelEx>> {
    fn from(value: BelongsTo<Option<E>>) -> Self {
        match value {
            BelongsTo::Loaded(model) => model,
            BelongsTo::Unloaded => None,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<E::ModelEx>>> for BelongsTo<E> {
    fn from(value: Option<Box<E::ModelEx>>) -> Self {
        match value {
            Some(model) => Self::Loaded(model),
            None => Self::Unloaded,
        }
    }
}

impl<E: EntityTrait> From<Option<Box<E::ModelEx>>> for BelongsTo<Option<E>> {
    fn from(value: Option<Box<E::ModelEx>>) -> Self {
        Self::Loaded(value)
    }
}

impl<E> PartialEq<Option<Box<E::ModelEx>>> for BelongsTo<E>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &Option<Box<E::ModelEx>>) -> bool {
        match (self, other) {
            (Self::Loaded(a), Some(b)) => a.as_ref() == b.as_ref(),
            (Self::Unloaded, None) => true,
            _ => false,
        }
    }
}

impl<E> PartialEq<Option<Box<E::ModelEx>>> for BelongsTo<Option<E>>
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

impl<E> PartialEq<BelongsTo<E>> for Option<Box<E::ModelEx>>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &BelongsTo<E>) -> bool {
        other == self
    }
}

impl<E> PartialEq<BelongsTo<Option<E>>> for Option<Box<E::ModelEx>>
where
    E: EntityTrait,
    E::ModelEx: PartialEq,
{
    fn eq(&self, other: &BelongsTo<Option<E>>) -> bool {
        other == self
    }
}

#[cfg(feature = "with-json")]
impl<T> serde::Serialize for BelongsTo<T>
where
    T: BelongsToCardinality,
    <T as BelongsToCardinality>::Loaded: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            BelongsTo::Unloaded => None,
            BelongsTo::Loaded(model) => Some(model),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "with-json")]
impl<'de, E> serde::Deserialize<'de> for BelongsTo<E>
where
    E: EntityTrait,
    E::ModelEx: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<E::ModelEx>>::deserialize(deserializer)? {
            Some(model) => Ok(BelongsTo::Loaded(Box::new(model))),
            None => Ok(BelongsTo::Unloaded),
        }
    }
}

#[cfg(feature = "with-json")]
impl<'de, E> serde::Deserialize<'de> for BelongsTo<Option<E>>
where
    E: EntityTrait,
    E::ModelEx: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match <Option<E::ModelEx>>::deserialize(deserializer)? {
            Some(model) => Ok(BelongsTo::Loaded(Some(Box::new(model)))),
            None => Ok(BelongsTo::Unloaded),
        }
    }
}
