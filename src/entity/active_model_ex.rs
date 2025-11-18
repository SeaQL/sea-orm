use crate::{ActiveModelTrait, EntityTrait, ModelTrait};
use core::ops::{Index, IndexMut};

/// Container for belongs_to or has_one relation
#[derive(Debug, Default, Clone)]
pub enum HasOneModel<E: EntityTrait> {
    /// Unspecified value, do nothing
    #[default]
    NotSet,
    /// Specify the value for the has one relation
    Set(Box<E::ActiveModelEx>),
}

/// Container for 1-N or M-N related Models
#[derive(Debug, Default, Clone)]
pub enum HasManyModel<E: EntityTrait> {
    /// Unspecified value, do nothing
    #[default]
    NotSet,
    /// Replace all items with this value set; delete leftovers
    Replace(Vec<E::ActiveModelEx>),
    /// Append new items to this has many relation; do not delete
    Append(Vec<E::ActiveModelEx>),
}

/// Action to perform on ActiveModel
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActiveModelAction {
    /// Insert
    Insert,
    /// Update
    Update,
    /// Insert the model if primary key is `NotSet`, update otherwise.
    /// Only works if the entity has auto increment primary key.
    Save,
}

impl<E> HasOneModel<E>
where
    E: EntityTrait,
{
    /// Construct a `HasOneModel::Set`
    pub fn set(model: E::ActiveModelEx) -> Self {
        Self::Set(model.into())
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
    pub fn as_mut(&mut self) -> Option<&mut E::ActiveModelEx> {
        match self {
            Self::Set(model) => Some(model),
            _ => None,
        }
    }

    /// Return true if the containing model is set and changed
    pub fn is_changed(&self) -> bool {
        match self {
            Self::Set(model) => model.is_changed(),
            _ => false,
        }
    }

    #[doc(hidden)]
    pub fn empty_slice(&self) -> &[E::ActiveModelEx] {
        &[]
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
    pub fn push(&mut self, model: E::ActiveModelEx) {
        match self {
            Self::Replace(models) | Self::Append(models) => models.push(model),
            Self::NotSet => {
                *self = Self::Append(vec![model]);
            }
        }
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
            if let Some(pk_item) = item.get_primary_key_value() {
                if pk_item == pk {
                    return true;
                }
            }
        }

        false
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
