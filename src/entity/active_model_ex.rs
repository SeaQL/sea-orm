use super::compound::{HasMany, HasOne};
use crate::{ActiveModelTrait, DbErr, EntityTrait, ModelTrait, TryIntoModel};
use core::ops::{Index, IndexMut};

/// State carried by a `belongs_to` or `has_one` field on an
/// [`ActiveModelEx`](crate::EntityTrait::ActiveModelEx). Mirrors the
/// `NotSet` / `Set` shape of [`ActiveValue`](crate::ActiveValue) but for a
/// related model.
///
/// ⚠️ **Unstable:** nested-`ActiveModel` relation mutation is exempt from semver — the
/// semantics of replacing or removing related records may change in a minor (2.x) release.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub enum HasOneModel<E: EntityTrait> {
    /// Field is absent; the related model is left as-is on save.
    #[default]
    NotSet,
    /// Field is being assigned to this related ActiveModel on save.
    Set(Box<E::ActiveModelEx>),
}

/// State carried by a `has_many` (or many-to-many) field on an
/// [`ActiveModelEx`](crate::EntityTrait::ActiveModelEx). Chooses between
/// "leave alone", "additive write", and "destructive replace" semantics.
///
/// ⚠️ **Unstable:** nested-`ActiveModel` relation mutation is exempt from semver — the
/// semantics of replacing or removing related records may change in a minor (2.x) release.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub enum HasManyModel<E: EntityTrait> {
    /// Field is absent; existing related models are left as-is on save.
    #[default]
    NotSet,
    /// Persist exactly this list of related models, deleting any existing
    /// children that are not in the list.
    Replace(Vec<E::ActiveModelEx>),
    /// Persist these related models alongside any existing children; never
    /// deletes.
    Append(Vec<E::ActiveModelEx>),
}

/// Which save operation an [`ActiveModel`](crate::ActiveModelTrait) is about
/// to perform — used by hooks and helpers that need to branch on the kind
/// of write.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActiveModelAction {
    /// `INSERT`.
    Insert,
    /// `UPDATE`.
    Update,
    /// Insert if the primary key is `NotSet`, otherwise update.
    /// Only meaningful for entities with an auto-increment primary key.
    Save,
}

impl<E> HasOneModel<E>
where
    E: EntityTrait,
{
    /// Construct a `HasOneModel::Set`
    pub fn set<AM: Into<E::ActiveModelEx>>(model: AM) -> Self {
        Self::Set(Box::new(model.into()))
    }

    /// Replace the inner Model
    pub fn replace<AM: Into<E::ActiveModelEx>>(&mut self, model: AM) {
        *self = Self::Set(Box::new(model.into()));
    }

    /// Take ownership of this model, leaving `NotSet` in place
    pub fn take(&mut self) -> Option<E::ActiveModelEx> {
        match std::mem::take(self) {
            Self::Set(model) => Some(*model),
            _ => None,
        }
    }

    /// Get a reference, if set
    pub fn as_ref(&self) -> Option<&E::ActiveModelEx> {
        match self {
            Self::Set(model) => Some(model),
            _ => None,
        }
    }

    /// Get a mutable reference, if set
    #[allow(clippy::should_implement_trait)]
    pub fn as_mut(&mut self) -> Option<&mut E::ActiveModelEx> {
        match self {
            Self::Set(model) => Some(model),
            _ => None,
        }
    }

    /// Return true if there is a model
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }

    /// Return true if self is NotSet
    pub fn is_not_set(&self) -> bool {
        matches!(self, Self::NotSet)
    }

    /// Return true if self is NotSet
    pub fn is_none(&self) -> bool {
        matches!(self, Self::NotSet)
    }

    /// Return true if the containing model is set and changed
    pub fn is_changed(&self) -> bool {
        match self {
            Self::Set(model) => model.is_changed(),
            _ => false,
        }
    }

    /// Convert into an `Option<ActiveModelEx>`
    pub fn into_option(self) -> Option<E::ActiveModelEx> {
        match self {
            Self::Set(model) => Some(*model),
            Self::NotSet => None,
        }
    }

    /// For type inference purpose
    #[doc(hidden)]
    pub fn empty_slice(&self) -> &[E::ActiveModelEx] {
        &[]
    }

    /// Convert this back to a `ModelEx` container
    pub fn try_into_model(self) -> Result<HasOne<E>, DbErr>
    where
        E::ActiveModelEx: TryIntoModel<E::ModelEx>,
    {
        Ok(match self {
            Self::Set(model) => HasOne::Loaded(Box::new((*model).try_into_model()?)),
            Self::NotSet => HasOne::Unloaded,
        })
    }
}

impl<E> PartialEq for HasOneModel<E>
where
    E: EntityTrait,
    E::ActiveModelEx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HasOneModel::NotSet, HasOneModel::NotSet) => true,
            (HasOneModel::Set(a), HasOneModel::Set(b)) => a == b,
            _ => false,
        }
    }
}

impl<E> PartialEq<Option<E::ActiveModelEx>> for HasOneModel<E>
where
    E: EntityTrait,
    E::ActiveModelEx: PartialEq,
{
    fn eq(&self, other: &Option<E::ActiveModelEx>) -> bool {
        match (self, other) {
            (HasOneModel::NotSet, None) => true,
            (HasOneModel::Set(a), Some(b)) => a.as_ref() == b,
            _ => false,
        }
    }
}

impl<E> Eq for HasOneModel<E>
where
    E: EntityTrait,
    E::ActiveModelEx: Eq,
{
}

impl<E> HasManyModel<E>
where
    E: EntityTrait,
{
    /// Take ownership of the models, leaving `NotSet` in place
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    /// Borrow models as slice
    pub fn as_slice(&self) -> &[E::ActiveModelEx] {
        match self {
            Self::Replace(models) | Self::Append(models) => models,
            Self::NotSet => &[],
        }
    }

    /// Get a mutable vec. If self is `NotSet`, convert to append.
    pub fn as_mut_vec(&mut self) -> &mut Vec<E::ActiveModelEx> {
        match self {
            Self::Replace(models) | Self::Append(models) => models,
            Self::NotSet => {
                *self = Self::Append(vec![]);
                self.as_mut_vec()
            }
        }
    }

    /// Consume self as vector
    pub fn into_vec(self) -> Vec<E::ActiveModelEx> {
        match self {
            Self::Replace(models) | Self::Append(models) => models,
            Self::NotSet => vec![],
        }
    }

    /// Returns an empty container of self
    pub fn empty_holder(&self) -> Self {
        match self {
            Self::Replace(_) => Self::Replace(vec![]),
            Self::Append(_) => Self::Append(vec![]),
            Self::NotSet => Self::NotSet,
        }
    }

    /// Push an item to self
    pub fn push<AM: Into<E::ActiveModelEx>>(&mut self, model: AM) -> &mut Self {
        let model = model.into();
        match self {
            Self::Replace(models) | Self::Append(models) => models.push(model),
            Self::NotSet => {
                *self = Self::Append(vec![model]);
            }
        }

        self
    }

    /// Push an item to self, but convert Replace to Append
    pub fn append<AM: Into<E::ActiveModelEx>>(&mut self, model: AM) -> &mut Self {
        self.convert_to_append().push(model)
    }

    /// Replace all items in this set
    pub fn replace_all<I>(&mut self, models: I) -> &mut Self
    where
        I: IntoIterator<Item = E::ActiveModelEx>,
    {
        *self = Self::Replace(models.into_iter().collect());
        self
    }

    /// Convert self to Append, if set
    pub fn convert_to_append(&mut self) -> &mut Self {
        match self.take() {
            Self::Replace(models) | Self::Append(models) => {
                *self = Self::Append(models);
            }
            Self::NotSet => {
                *self = Self::NotSet;
            }
        }

        self
    }

    /// Reset self to NotSet
    pub fn not_set(&mut self) {
        *self = Self::NotSet;
    }

    /// If self is `Replace`
    pub fn is_replace(&self) -> bool {
        matches!(self, Self::Replace(_))
    }

    /// If self is `Append`
    pub fn is_append(&self) -> bool {
        matches!(self, Self::Append(_))
    }

    /// Return true if self is `Replace` or any containing model is changed
    pub fn is_changed(&self) -> bool {
        match self {
            Self::Replace(_) => true,
            Self::Append(models) => models.iter().any(|model| model.is_changed()),
            Self::NotSet => false,
        }
    }

    /// Find within the models by primary key, return true if found
    pub fn find(&self, model: &E::Model) -> bool {
        let pk = model.get_primary_key_value();

        for item in self.as_slice() {
            if let Some(pk_item) = item.get_primary_key_value()
                && pk_item == pk
            {
                return true;
            }
        }

        false
    }

    /// Convert this back to a `ModelEx` container
    pub fn try_into_model(self) -> Result<HasMany<E>, DbErr>
    where
        E::ActiveModelEx: TryIntoModel<E::ModelEx>,
    {
        Ok(match self {
            Self::Replace(models) | Self::Append(models) => HasMany::Loaded(
                models
                    .into_iter()
                    .map(|t| t.try_into_model())
                    .collect::<Result<Vec<_>, DbErr>>()?,
            ),
            Self::NotSet => HasMany::Unloaded,
        })
    }
}

impl<E> PartialEq for HasManyModel<E>
where
    E: EntityTrait,
    E::ActiveModelEx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HasManyModel::NotSet, HasManyModel::NotSet) => true,
            (HasManyModel::Replace(a), HasManyModel::Replace(b)) => a == b,
            (HasManyModel::Append(a), HasManyModel::Append(b)) => a == b,
            _ => false,
        }
    }
}

impl<E> Eq for HasManyModel<E>
where
    E: EntityTrait,
    E::ActiveModelEx: Eq,
{
}

impl<E: EntityTrait> From<HasManyModel<E>> for Option<Vec<E::ActiveModelEx>> {
    fn from(value: HasManyModel<E>) -> Self {
        match value {
            HasManyModel::NotSet => None,
            HasManyModel::Replace(models) | HasManyModel::Append(models) => Some(models),
        }
    }
}

impl<E: EntityTrait> Index<usize> for HasManyModel<E> {
    type Output = E::ActiveModelEx;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            HasManyModel::NotSet => {
                panic!("index out of bounds: the HasManyModel is NotSet (index: {index})")
            }
            HasManyModel::Replace(models) | HasManyModel::Append(models) => models.index(index),
        }
    }
}

impl<E: EntityTrait> IndexMut<usize> for HasManyModel<E> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            HasManyModel::NotSet => {
                panic!("index out of bounds: the HasManyModel is NotSet (index: {index})")
            }
            HasManyModel::Replace(models) | HasManyModel::Append(models) => models.index_mut(index),
        }
    }
}

impl<E: EntityTrait> IntoIterator for HasManyModel<E> {
    type Item = E::ActiveModelEx;
    type IntoIter = std::vec::IntoIter<E::ActiveModelEx>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            HasManyModel::Replace(models) | HasManyModel::Append(models) => models.into_iter(),
            HasManyModel::NotSet => Vec::new().into_iter(),
        }
    }
}

/// Converts from a set of models into `Append`, which performs non-destructive action
impl<E: EntityTrait> From<Vec<E::ActiveModelEx>> for HasManyModel<E> {
    fn from(value: Vec<E::ActiveModelEx>) -> Self {
        HasManyModel::Append(value)
    }
}
