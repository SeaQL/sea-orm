use crate::{ActiveModelBehavior, ActiveModelTrait, ActiveValue, DbErr, EntityTrait, Value};
use std::marker::PhantomData;

/// A dummy ActiveModel for read-only entities (Views).
/// All write operations will return errors or panic at runtime.
#[derive(Clone, Debug)]
pub struct NeverActiveModel<E> {
    _marker: PhantomData<E>,
}

impl<E> Default for NeverActiveModel<E> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<E: EntityTrait> ActiveModelTrait for NeverActiveModel<E> {
    type Entity = E;

    fn take(&mut self, _c: E::Column) -> ActiveValue<Value> {
        panic!("Cannot modify a read-only view: attempted to take column value")
    }

    fn get(&self, _c: E::Column) -> ActiveValue<Value> {
        // Return NotSet for all columns - views don't track active values
        ActiveValue::NotSet
    }

    fn set_if_not_equals(&mut self, _c: E::Column, _v: Value) {
        panic!("Cannot modify a read-only view: attempted to set column value")
    }

    fn try_set(&mut self, _c: E::Column, _v: Value) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Cannot modify a read-only view: attempted to set column value".to_owned(),
        ))
    }

    fn not_set(&mut self, _c: E::Column) {}

    fn is_not_set(&self, _c: E::Column) -> bool {
        true
    }

    fn default() -> Self {
        <Self as Default>::default()
    }

    fn default_values() -> Self {
        <Self as Default>::default()
    }

    fn reset(&mut self, _c: E::Column) {}
}

impl<E: EntityTrait> ActiveModelBehavior for NeverActiveModel<E> {}
