use crate::{
    error::*, DatabaseConnection, DeleteResult, EntityTrait, Iterable, PrimaryKeyToColumn,
    PrimaryKeyTrait, Value,
};
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

pub trait ActiveModelTrait: Clone + Debug {
    type Entity: EntityTrait;

    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value);

    fn unset(&mut self, c: <Self::Entity as EntityTrait>::Column);

    fn is_unset(&self, c: <Self::Entity as EntityTrait>::Column) -> bool;

    fn default() -> Self;

    // below is not yet possible. right now we define these methods in DeriveActiveModel
    // fn save(self, db: &DatabaseConnection) -> impl Future<Output = Result<Self, DbErr>>;
    // fn delete(self, db: &DatabaseConnection) -> impl Future<Output = Result<DeleteResult, DbErr>>;
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

/// Insert the model if primary key is unset, update otherwise.
/// Only works if the entity has auto increment primary key.
pub async fn save_active_model<A, E>(mut am: A, db: &DatabaseConnection) -> Result<A, DbErr>
where
    A: ActiveModelBehavior + ActiveModelTrait<Entity = E>,
    E::Model: IntoActiveModel<A>,
    E: EntityTrait,
{
    am = ActiveModelBehavior::before_save(am);
    let mut is_update = true;
    for key in E::PrimaryKey::iter() {
        let col = key.into_column();
        if am.is_unset(col) {
            is_update = false;
            break;
        }
    }
    if !is_update {
        am = insert_and_select_active_model::<A, E>(am, db).await?;
    } else {
        am = update_active_model::<A, E>(am, db).await?;
    }
    am = ActiveModelBehavior::after_save(am);
    Ok(am)
}

async fn insert_and_select_active_model<A, E>(am: A, db: &DatabaseConnection) -> Result<A, DbErr>
where
    A: ActiveModelTrait<Entity = E>,
    E::Model: IntoActiveModel<A>,
    E: EntityTrait,
{
    let exec = E::insert(am).exec(db);
    let res = exec.await?;
    // TODO: if the entity does not have auto increment primary key, then last_insert_id is a wrong value
    if <E::PrimaryKey as PrimaryKeyTrait>::auto_increment() && res.last_insert_id != 0 {
        let find = E::find_by_id(res.last_insert_id).one(db);
        let found = find.await;
        let model: Option<E::Model> = found?;
        match model {
            Some(model) => Ok(model.into_active_model()),
            None => Err(DbErr::Exec(format!(
                "Failed to find inserted item: {} {}",
                E::default().to_string(),
                res.last_insert_id
            ))),
        }
    } else {
        Ok(A::default())
    }
}

async fn update_active_model<A, E>(am: A, db: &DatabaseConnection) -> Result<A, DbErr>
where
    A: ActiveModelTrait<Entity = E>,
    E: EntityTrait,
{
    let exec = E::update(am).exec(db);
    exec.await
}

/// Delete an active model by its primary key
pub async fn delete_active_model<A, E>(
    mut am: A,
    db: &DatabaseConnection,
) -> Result<DeleteResult, DbErr>
where
    A: ActiveModelBehavior + ActiveModelTrait<Entity = E>,
    E: EntityTrait,
{
    am = ActiveModelBehavior::before_delete(am);
    let exec = E::delete(am).exec(db);
    exec.await
}
