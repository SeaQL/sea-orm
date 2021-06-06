use crate::{Database, EntityTrait, ExecErr, Iterable, PrimaryKeyToColumn, Value};
use std::fmt::Debug;

#[derive(Clone, Debug, Default)]
pub struct ActiveValue<V>
where
    V: Into<Value> + Default,
{
    value: V,
    state: ActiveValueState,
}

pub type Val<V> = ActiveValue<V>;

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

pub trait OneOrManyActiveModel<A>
where
    A: ActiveModelTrait,
{
    fn is_one() -> bool;
    fn get_one(self) -> A;

    fn is_many() -> bool;
    fn get_many(self) -> Vec<A>;
}

#[doc(hidden)]
pub fn unchanged_active_value_not_intended_for_public_use<V>(value: V) -> ActiveValue<V>
where
    V: Into<Value> + Default,
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
}

/// Behaviors for users to override
pub trait ActiveModelBehavior: ActiveModelTrait {
    /// Create a new ActiveModel with default values. Also used by `Default::default()`.
    fn new() -> Self {
        <Self as ActiveModelTrait>::default()
    }

    /// Will be called before saving to database
    fn before_save(self) -> Self {
        self
    }

    /// Will be called after saving to database
    fn after_save(self) -> Self {
        self
    }
}

impl<V> ActiveValue<V>
where
    V: Into<Value> + Default,
{
    pub fn set(value: V) -> Self {
        Self {
            value,
            state: ActiveValueState::Set,
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self.state, ActiveValueState::Set)
    }

    pub(crate) fn unchanged(value: V) -> Self {
        Self {
            value,
            state: ActiveValueState::Unchanged,
        }
    }

    pub fn is_unchanged(&self) -> bool {
        matches!(self.state, ActiveValueState::Unchanged)
    }

    pub fn unset() -> Self {
        Self {
            value: V::default(),
            state: ActiveValueState::Unset,
        }
    }

    pub fn is_unset(&self) -> bool {
        matches!(self.state, ActiveValueState::Unset)
    }

    pub fn take(&mut self) -> V {
        self.state = ActiveValueState::Unset;
        std::mem::take(&mut self.value)
    }

    pub fn unwrap(self) -> V {
        self.value
    }

    pub fn into_value(self) -> Value {
        self.value.into()
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
    V: Into<Value> + Default,
{
    fn as_ref(&self) -> &V {
        &self.value
    }
}

impl<A> OneOrManyActiveModel<A> for A
where
    A: ActiveModelTrait,
{
    fn is_one() -> bool {
        true
    }
    fn get_one(self) -> A {
        self
    }

    fn is_many() -> bool {
        false
    }
    fn get_many(self) -> Vec<A> {
        panic!("not many")
    }
}

impl<A> OneOrManyActiveModel<A> for Vec<A>
where
    A: ActiveModelTrait,
{
    fn is_one() -> bool {
        false
    }
    fn get_one(self) -> A {
        panic!("not one")
    }

    fn is_many() -> bool {
        true
    }
    fn get_many(self) -> Vec<A> {
        self
    }
}

/// Insert the model if primary key is unset, update otherwise
pub async fn save_active_model<A, E>(mut am: A, db: &Database) -> Result<A, ExecErr>
where
    A: ActiveModelBehavior + ActiveModelTrait<Entity = E> + From<E::Model>,
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

async fn insert_and_select_active_model<A, E>(am: A, db: &Database) -> Result<A, ExecErr>
where
    A: ActiveModelTrait<Entity = E> + From<E::Model>,
    E: EntityTrait,
{
    let exec = E::insert(am).exec(db);
    let res = exec.await?;
    if res.last_insert_id != 0 {
        let find = E::find_by(res.last_insert_id).one(db);
        let res = find.await;
        let model: Option<E::Model> = res.map_err(|_| ExecErr)?;
        match model {
            Some(model) => Ok(model.into()),
            None => Err(ExecErr),
        }
    } else {
        Ok(A::default())
    }
}

async fn update_active_model<A, E>(am: A, db: &Database) -> Result<A, ExecErr>
where
    A: ActiveModelTrait<Entity = E>,
    E: EntityTrait,
{
    let exec = E::update(am).exec(db);
    exec.await
}
