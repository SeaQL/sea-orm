use crate::{
    error::*, DatabaseConnection, DeleteResult, EntityTrait, Iterable, PrimaryKeyToColumn,
    PrimaryKeyTrait, Value,
};
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
pub struct ActiveValue<V>
where
    V: Into<Value>,
{
    value: Option<V>,
    state: ActiveValueState,
}

#[allow(non_snake_case)]
pub fn Set<V>(v: V) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::set(v)
}

#[allow(non_snake_case)]
pub fn Unset<V>(_: Option<bool>) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::unset()
}

#[derive(Clone, Debug)]
enum ActiveValueState {
    Set,
    Unchanged,
    Unset,
}

impl Default for ActiveValueState {
    fn default() -> Self {
        Self::Unset
    }
}

#[doc(hidden)]
pub fn unchanged_active_value_not_intended_for_public_use<V>(value: V) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::unchanged(value)
}

#[async_trait]
pub trait ActiveModelTrait: Clone + Debug {
    type Entity: EntityTrait;

    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value);

    fn unset(&mut self, c: <Self::Entity as EntityTrait>::Column);

    fn is_unset(&self, c: <Self::Entity as EntityTrait>::Column) -> bool;

    fn default() -> Self;

    async fn insert(self, db: &DatabaseConnection) -> Result<Self, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
    {
        let am = self;
        let exec = <Self::Entity as EntityTrait>::insert(am).exec(db);
        let res = exec.await?;
        // Assume valid last_insert_id is not equals to Default::default()
        if res.last_insert_id
            != <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType::default()
        {
            let found = <Self::Entity as EntityTrait>::find_by_id(res.last_insert_id)
                .one(db)
                .await?;
            match found {
                Some(model) => Ok(model.into_active_model()),
                None => Err(DbErr::Exec("Failed to find inserted item".to_owned())),
            }
        } else {
            Ok(Self::default())
        }
    }

    async fn update(self, db: &DatabaseConnection) -> Result<Self, DbErr> {
        let exec = Self::Entity::update(self).exec(db);
        exec.await
    }

    /// Insert the model if primary key is unset, update otherwise.
    /// Only works if the entity has auto increment primary key.
    async fn save(self, db: &DatabaseConnection) -> Result<Self, DbErr>
    where
        Self: ActiveModelBehavior,
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
    {
        let mut am = self;
        am = ActiveModelBehavior::before_save(am);
        let mut is_update = true;
        for key in <Self::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            if am.is_unset(col) {
                is_update = false;
                break;
            }
        }
        if !is_update {
            am = am.insert(db).await?;
        } else {
            am = am.update(db).await?;
        }
        am = ActiveModelBehavior::after_save(am);
        Ok(am)
    }

    /// Delete an active model by its primary key
    async fn delete(self, db: &DatabaseConnection) -> Result<DeleteResult, DbErr>
    where
        Self: ActiveModelBehavior,
    {
        let mut am = self;
        am = ActiveModelBehavior::before_delete(am);
        let exec = Self::Entity::delete(am).exec(db);
        exec.await
    }
}

/// Behaviors for users to override
pub trait ActiveModelBehavior: ActiveModelTrait {
    /// Create a new ActiveModel with default values. Also used by `Default::default()`.
    fn new() -> Self {
        <Self as ActiveModelTrait>::default()
    }

    /// Will be called before saving
    fn before_save(self) -> Self {
        self
    }

    /// Will be called after saving
    fn after_save(self) -> Self {
        self
    }

    /// Will be called before deleting
    fn before_delete(self) -> Self {
        self
    }
}

pub trait IntoActiveModel<A>
where
    A: ActiveModelTrait,
{
    fn into_active_model(self) -> A;
}

impl<A> IntoActiveModel<A> for A
where
    A: ActiveModelTrait,
{
    fn into_active_model(self) -> A {
        self
    }
}

impl<V> ActiveValue<V>
where
    V: Into<Value>,
{
    pub fn set(value: V) -> Self {
        Self {
            value: Some(value),
            state: ActiveValueState::Set,
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self.state, ActiveValueState::Set)
    }

    pub(crate) fn unchanged(value: V) -> Self {
        Self {
            value: Some(value),
            state: ActiveValueState::Unchanged,
        }
    }

    pub fn is_unchanged(&self) -> bool {
        matches!(self.state, ActiveValueState::Unchanged)
    }

    pub fn unset() -> Self {
        Self {
            value: None,
            state: ActiveValueState::Unset,
        }
    }

    pub fn is_unset(&self) -> bool {
        matches!(self.state, ActiveValueState::Unset)
    }

    pub fn take(&mut self) -> V {
        self.state = ActiveValueState::Unset;
        self.value.take().unwrap()
    }

    pub fn unwrap(self) -> V {
        self.value.unwrap()
    }

    pub fn into_value(self) -> Value {
        self.value.unwrap().into()
    }

    pub fn into_wrapped_value(self) -> ActiveValue<Value> {
        match self.state {
            ActiveValueState::Set => ActiveValue::set(self.into_value()),
            ActiveValueState::Unchanged => ActiveValue::unchanged(self.into_value()),
            ActiveValueState::Unset => ActiveValue::unset(),
        }
    }
}

impl<V> std::convert::AsRef<V> for ActiveValue<V>
where
    V: Into<Value>,
{
    fn as_ref(&self) -> &V {
        self.value.as_ref().unwrap()
    }
}

impl<V> PartialEq for ActiveValue<V>
where
    V: Into<Value> + std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value.as_ref() == other.value.as_ref()
    }
}
