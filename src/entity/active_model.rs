use crate::{ColumnTrait, EntityTrait, Value};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum Action<V> {
    Set(V),
    Unset,
}

impl<V> Default for Action<V> {
    fn default() -> Self {
        Self::Unset
    }
}

impl<V> Action<V>
where
    V: Into<Value>,
{
    pub fn into_action_value(self) -> Action<Value> {
        match self {
            Self::Set(v) => Action::Set(v.into()),
            Self::Unset => Action::Unset,
        }
    }
}

pub trait ActiveModelOf<E>
where
    E: EntityTrait,
{
}

pub trait ActiveModelTrait: Clone + Debug {
    type Column: ColumnTrait;

    fn take(&mut self, c: Self::Column) -> Action<Value>;

    fn get(&self, c: Self::Column) -> Action<Value>;

    fn set(&mut self, c: Self::Column, v: Value);

    fn unset(&mut self, c: Self::Column);
}
