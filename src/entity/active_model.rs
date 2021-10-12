use crate::{
    error::*, ConnectionTrait, DeleteResult, EntityTrait, Iterable, PrimaryKeyToColumn, Value,
};
use async_trait::async_trait;
use sea_query::ValueTuple;
use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
pub struct ActiveValue<V>
where
    V: Into<Value>,
{
    // Don't want to call ActiveValue::unwrap() and cause panic
    pub(self) value: Option<V>,
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

    #[allow(clippy::question_mark)]
    fn get_primary_key_value(&self) -> Option<ValueTuple> {
        let mut cols = <Self::Entity as EntityTrait>::PrimaryKey::iter();
        macro_rules! next {
            () => {
                if let Some(col) = cols.next() {
                    if let Some(val) = self.get(col.into_column()).value {
                        val
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            };
        }
        match <Self::Entity as EntityTrait>::PrimaryKey::iter().count() {
            1 => {
                let s1 = next!();
                Some(ValueTuple::One(s1))
            }
            2 => {
                let s1 = next!();
                let s2 = next!();
                Some(ValueTuple::Two(s1, s2))
            }
            3 => {
                let s1 = next!();
                let s2 = next!();
                let s3 = next!();
                Some(ValueTuple::Three(s1, s2, s3))
            }
            _ => panic!("The arity cannot be larger than 3"),
        }
    }

    async fn insert<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        Self: ActiveModelBehavior,
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        C: ConnectionTrait<'a>,
        Self: 'a,
    {
        let am = ActiveModelBehavior::before_save(self, true, db)?;
        let res = <Self::Entity as EntityTrait>::insert(am).exec(db).await?;
        let found = <Self::Entity as EntityTrait>::find_by_id(res.last_insert_id)
            .one(db)
            .await?;
        match found {
            Some(model) => Ok(model.into_active_model()),
            None => Err(DbErr::Exec("Failed to find inserted item".to_owned())),
        }
    }

    async fn update<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        C: ConnectionTrait<'a>,
        Self: 'a,
    {
        let am = ActiveModelBehavior::before_save(self, false, db)?;
        let am = Self::Entity::update(am).exec(db).await?;
        ActiveModelBehavior::after_save(am, false, db)
    }

    /// Insert the model if primary key is unset, update otherwise.
    /// Only works if the entity has auto increment primary key.
    async fn save<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        Self: ActiveModelBehavior + 'a,
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        C: ConnectionTrait<'a>,
    {
        let mut am = self;
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
        Ok(am)
    }

    /// Delete an active model by its primary key
    async fn delete<'a, C>(self, db: &'a C) -> Result<DeleteResult, DbErr>
    where
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait<'a>,
    {
        let am = ActiveModelBehavior::before_delete(self, db)?;
        let am_clone = am.clone();
        let delete_res = Self::Entity::delete(am).exec(db).await?;
        ActiveModelBehavior::after_delete(am_clone, db)?;
        Ok(delete_res)
    }
}

/// Behaviors for users to override
#[allow(unused_variables)]
pub trait ActiveModelBehavior: ActiveModelTrait {
    /// Create a new ActiveModel with default values. Also used by `Default::default()`.
    fn new() -> Self {
        <Self as ActiveModelTrait>::default()
    }

    /// Will be called before saving
    fn before_save(self, insert: bool, db: &DatabaseConnection) -> Result<Self, DbErr> {
        Ok(self)
    }

    /// Will be called after saving
    fn after_save(self, insert: bool, db: &DatabaseConnection) -> Result<Self, DbErr> {
        Ok(self)
    }

    /// Will be called before deleting
    fn before_delete(self, db: &DatabaseConnection) -> Result<Self, DbErr> {
        Ok(self)
    }

    /// Will be called after deleting
    fn after_delete(self, db: &DatabaseConnection) -> Result<Self, DbErr> {
        Ok(self)
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

    pub fn take(&mut self) -> Option<V> {
        self.state = ActiveValueState::Unset;
        self.value.take()
    }

    pub fn unwrap(self) -> V {
        self.value.unwrap()
    }

    pub fn into_value(self) -> Option<Value> {
        self.value.map(Into::into)
    }

    pub fn into_wrapped_value(self) -> ActiveValue<Value> {
        match self.state {
            ActiveValueState::Set => ActiveValue::set(self.into_value().unwrap()),
            ActiveValueState::Unchanged => ActiveValue::unchanged(self.into_value().unwrap()),
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
