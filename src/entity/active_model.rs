use std::fmt::Debug;
use crate::{ColumnTrait, ModelTrait, Value};

#[derive(Clone, Debug)]
pub enum Action<V> {
    Set(V),
    Unset,
}

impl<V> Action<V> where V: Into<Value> {
    pub fn into_action_value(self) -> Action<Value> {
        match self {
            Self::Set(v) => Action::Set(v.into()),
            Self::Unset => Action::Unset,
        }
    }
}

pub trait ActiveModelOf<M>
where
    M: ModelTrait,
{
    fn from_model(m: M) -> Self;
}

pub trait ActiveModelTrait: Clone + Debug {
    type Column: ColumnTrait;

    fn get(&self, c: Self::Column) -> Action<Value>;

    fn set(&mut self, c: Self::Column, v: Value);

    fn unset(&mut self, c: Self::Column);
}