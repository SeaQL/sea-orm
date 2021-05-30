use crate::{ColumnTrait, EntityTrait, Value};
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

pub trait ActiveModelOf<E>
where
    E: EntityTrait,
{
}

pub trait ActiveModelTrait: Clone + Debug + Default {
    type Column: ColumnTrait;

    fn take(&mut self, c: Self::Column) -> ActiveValue<Value>;

    fn get(&self, c: Self::Column) -> ActiveValue<Value>;

    fn set(&mut self, c: Self::Column, v: Value);

    fn unset(&mut self, c: Self::Column);
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
            ActiveValueState::Unchanged => ActiveValue::set(self.into_value()),
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
