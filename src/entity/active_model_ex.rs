use crate::EntityTrait;

/// Container for belongs_to or has_one relation
#[derive(Debug, Default, Clone)]
pub enum HasOneModel<E: EntityTrait> {
    /// Unspecified value, do nothing
    #[default]
    NotSet,
    /// Specify the value for the has one relation
    Set(Box<<E as EntityTrait>::ActiveModelEx>),
}

/// Container for 1-N or M-N related Models
#[derive(Debug, Default, Clone)]
pub enum HasManyModel<E: EntityTrait> {
    /// Unspecified value, do nothing
    #[default]
    NotSet,
    /// Replace all items with this value set
    Replace(Vec<<E as EntityTrait>::ActiveModelEx>),
    /// Append items to this has many relation
    Append(Vec<<E as EntityTrait>::ActiveModelEx>),
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
    pub fn set(model: <E as EntityTrait>::ActiveModelEx) -> Self {
        Self::Set(model.into())
    }

    /// Take ownership of this model, leaving `NotSet` in place
    pub fn take(&mut self) -> Option<<E as EntityTrait>::ActiveModelEx> {
        match std::mem::take(self) {
            Self::Set(model) => Some(*model),
            _ => None,
        }
    }

    /// Get a mutable reference, if set
    pub fn as_mut(&mut self) -> Option<&mut <E as EntityTrait>::ActiveModelEx> {
        match self {
            Self::Set(model) => Some(model),
            _ => None,
        }
    }
}

impl<E> PartialEq for HasOneModel<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ActiveModelEx: PartialEq,
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
    <E as EntityTrait>::ActiveModelEx: Eq,
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
    pub fn as_slice(&self) -> &[<E as EntityTrait>::ActiveModelEx] {
        match self {
            Self::Replace(models) | Self::Append(models) => models,
            Self::NotSet => &[],
        }
    }

    /// Consume self as vector
    pub fn into_vec(self) -> Vec<<E as EntityTrait>::ActiveModelEx> {
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
    ///
    /// # Panics
    ///
    /// Panic if self is `NotSet`.
    pub fn push(&mut self, model: <E as EntityTrait>::ActiveModelEx) {
        match self {
            Self::Replace(models) | Self::Append(models) => models.push(model),
            Self::NotSet => panic!("Cannot push: self is NotSet"),
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
}

impl<E> PartialEq for HasManyModel<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ActiveModelEx: PartialEq,
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
    <E as EntityTrait>::ActiveModelEx: Eq,
{
}
