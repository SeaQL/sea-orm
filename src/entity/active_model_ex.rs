use crate::EntityTrait;

/// Container for belongs_to or has_one relation
#[derive(Debug, Default, Clone)]
pub enum HasOneModel<E: EntityTrait> {
    /// Unspecified value, do nothing
    #[default]
    NotSet,
    /// Clear the has one relation
    SetNone,
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

impl<E> PartialEq for HasOneModel<E>
where
    E: EntityTrait,
    <E as EntityTrait>::ActiveModelEx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HasOneModel::NotSet, HasOneModel::NotSet) => true,
            (HasOneModel::SetNone, HasOneModel::SetNone) => true,
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
