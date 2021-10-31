use crate::{
    error::*, ConnectionTrait, DeleteResult, EntityTrait, Iterable, PrimaryKeyToColumn, Value,
};
use async_trait::async_trait;
use sea_query::{Nullable, ValueTuple};
use std::fmt::Debug;

/// Defines a value from an ActiveModel and its state.
/// The field `value` takes in an [Option] type where `Option::Some(V)` , with `V` holding
/// the value that operations like `UPDATE` are being performed on and
/// the `state` field is either `ActiveValueState::Set` or `ActiveValueState::Unchanged`.
/// [Option::None] in the `value` field indicates no value being performed by an operation
/// and that the `state` field of the [ActiveValue] is set to `ActiveValueState::Unset` .
/// #### Example snippet
/// ```no_run
/// // The code snipped below does an UPDATE operation on a [ActiveValue]
/// // yielding the the SQL statement ` r#"UPDATE "fruit" SET "name" = 'Orange' WHERE "fruit"."id" = 1"# `
///
/// use sea_orm::tests_cfg::{cake, fruit};
/// use sea_orm::{entity::*, query::*, DbBackend};
///
/// Update::one(fruit::ActiveModel {
///     id: ActiveValue::set(1),
///     name: ActiveValue::set("Orange".to_owned()),
///     cake_id: ActiveValue::unset(),
/// })
/// .build(DbBackend::Postgres)
/// .to_string();
/// ```
#[derive(Clone, Debug, Default)]
pub struct ActiveValue<V>
where
    V: Into<Value>,
{
    value: Option<V>,
    state: ActiveValueState,
}

/// Defines a set operation on an [ActiveValue]
#[allow(non_snake_case)]
pub fn Set<V>(v: V) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::set(v)
}

/// Defines an unset operation on an [ActiveValue]
#[allow(non_snake_case)]
pub fn Unset<V>(_: Option<bool>) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::unset()
}

// Defines the state of an [ActiveValue]
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

/// Enforces a set of constraints on any type performing an Create, Update or Delete operation.
/// The type must also implement the [EntityTrait].
/// See module level docs [crate::entity] for a full example
#[async_trait]
pub trait ActiveModelTrait: Clone + Debug {
    /// Enforce the type to the constraints of the [EntityTrait]
    type Entity: EntityTrait;

    /// Get a mutable [ActiveValue] from an ActiveModel
    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    /// Get a immutable [ActiveValue] from an ActiveModel
    fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    /// Set the Value into an ActiveModel
    fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value);

    /// Set the state of an [ActiveValue] to the Unset state
    fn unset(&mut self, c: <Self::Entity as EntityTrait>::Column);

    /// Check the state of a [ActiveValue]
    fn is_unset(&self, c: <Self::Entity as EntityTrait>::Column) -> bool;

    /// The default implementation of the ActiveModel
    fn default() -> Self;

    /// Get the primary key of the ActiveModel
    #[allow(clippy::question_mark)]
    fn get_primary_key_value(&self) -> Option<ValueTuple> {
        let mut cols = <Self::Entity as EntityTrait>::PrimaryKey::iter();
        macro_rules! next {
            () => {
                if let Some(col) = cols.next() {
                    if let Some(val) = self.get(col.into_column()).into_value() {
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

    /// Perform an `INSERT` operation on the ActiveModel
    async fn insert<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait<'a>,
    {
        let am = ActiveModelBehavior::before_save(self, true)?;
        let res = <Self::Entity as EntityTrait>::insert(am).exec(db).await?;
        let found = <Self::Entity as EntityTrait>::find_by_id(res.last_insert_id)
            .one(db)
            .await?;
        let am = match found {
            Some(model) => model.into_active_model(),
            None => return Err(DbErr::Exec("Failed to find inserted item".to_owned())),
        };
        ActiveModelBehavior::after_save(am, true)
    }

    /// Perform the `UPDATE` operation on an ActiveModel
    async fn update<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait<'a>,
    {
        let am = ActiveModelBehavior::before_save(self, false)?;
        let am = Self::Entity::update(am).exec(db).await?;
        ActiveModelBehavior::after_save(am, false)
    }

    /// Insert the model if primary key is unset, update otherwise.
    /// Only works if the entity has auto increment primary key.
    async fn save<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
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
        let am = ActiveModelBehavior::before_delete(self)?;
        let am_clone = am.clone();
        let delete_res = Self::Entity::delete(am).exec(db).await?;
        ActiveModelBehavior::after_delete(am_clone)?;
        Ok(delete_res)
    }
}

/// Enforce a set of constraints to a override the ActiveModel behavior
/// Behaviors for users to override.
/// The type must also implement the [ActiveModelTrait]
///
/// ### Example
/// ```ignore
/// use sea_orm::entity::prelude::*;
///
///  // Use [DeriveEntity] to derive the EntityTrait automatically
/// #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
/// pub struct Entity;
///
/// /// The [EntityName] describes the name of a table
/// impl EntityName for Entity {
///     fn table_name(&self) -> &str {
///         "cake"
///     }
/// }
///
/// // Derive the ActiveModel
/// #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
/// pub struct Model {
///     pub id: i32,
///     pub name: String,
/// }
///
/// impl ActiveModelBehavior for ActiveModel {}
/// ```
/// See module level docs [crate::entity] for a full example
#[allow(unused_variables)]
pub trait ActiveModelBehavior: ActiveModelTrait {
    /// Create a new ActiveModel with default values. Also used by `Default::default()`.
    fn new() -> Self {
        <Self as ActiveModelTrait>::default()
    }

    /// Will be called before saving
    fn before_save(self, insert: bool) -> Result<Self, DbErr> {
        Ok(self)
    }

    /// Will be called after saving
    fn after_save(self, insert: bool) -> Result<Self, DbErr> {
        Ok(self)
    }

    /// Will be called before deleting
    fn before_delete(self) -> Result<Self, DbErr> {
        Ok(self)
    }

    /// Will be called after deleting
    fn after_delete(self) -> Result<Self, DbErr> {
        Ok(self)
    }
}

/// Enforce constraints for conversion to  an ActiveModel
pub trait IntoActiveModel<A>
where
    A: ActiveModelTrait,
{
    /// Method to call to perform the conversion
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

/// Constraints to perform the conversion of a type into an [ActiveValue]
pub trait IntoActiveValue<V>
where
    V: Into<Value>,
{
    /// Method to perform the conversion
    fn into_active_value(self) -> ActiveValue<V>;
}

macro_rules! impl_into_active_value {
    ($ty: ty, $fn: ident) => {
        impl IntoActiveValue<$ty> for $ty {
            fn into_active_value(self) -> ActiveValue<$ty> {
                $fn(self)
            }
        }

        impl IntoActiveValue<Option<$ty>> for Option<$ty> {
            fn into_active_value(self) -> ActiveValue<Option<$ty>> {
                match self {
                    Some(value) => Set(Some(value)),
                    None => Unset(None),
                }
            }
        }

        impl IntoActiveValue<Option<$ty>> for Option<Option<$ty>> {
            fn into_active_value(self) -> ActiveValue<Option<$ty>> {
                match self {
                    Some(value) => Set(value),
                    None => Unset(None),
                }
            }
        }
    };
}

impl_into_active_value!(bool, Set);
impl_into_active_value!(i8, Set);
impl_into_active_value!(i16, Set);
impl_into_active_value!(i32, Set);
impl_into_active_value!(i64, Set);
impl_into_active_value!(u8, Set);
impl_into_active_value!(u16, Set);
impl_into_active_value!(u32, Set);
impl_into_active_value!(u64, Set);
impl_into_active_value!(f32, Set);
impl_into_active_value!(f64, Set);
impl_into_active_value!(&'static str, Set);
impl_into_active_value!(String, Set);

#[cfg(feature = "with-json")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-json")))]
impl_into_active_value!(crate::prelude::Json, Set);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::Date, Set);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::Time, Set);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTime, Set);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTimeWithTimeZone, Set);

#[cfg(feature = "with-rust_decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-rust_decimal")))]
impl_into_active_value!(crate::prelude::Decimal, Set);

#[cfg(feature = "with-uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-uuid")))]
impl_into_active_value!(crate::prelude::Uuid, Set);

impl<V> ActiveValue<V>
where
    V: Into<Value>,
{
    /// Set the value of an [ActiveValue] and also set its state to `ActiveValueState::Set`
    pub fn set(value: V) -> Self {
        Self {
            value: Some(value),
            state: ActiveValueState::Set,
        }
    }

    /// Check if the state of an [ActiveValue] is `ActiveValueState::Set` which returns true
    pub fn is_set(&self) -> bool {
        matches!(self.state, ActiveValueState::Set)
    }

    pub(crate) fn unchanged(value: V) -> Self {
        Self {
            value: Some(value),
            state: ActiveValueState::Unchanged,
        }
    }

    /// Check if the status of the [ActiveValue] is `ActiveValueState::Unchanged`
    /// which returns `true` if it is
    pub fn is_unchanged(&self) -> bool {
        matches!(self.state, ActiveValueState::Unchanged)
    }

    /// Set the `value` field of the ActiveModel to [Option::None] and the
    /// `state` field to `ActiveValueState::Unset`
    pub fn unset() -> Self {
        Self {
            value: None,
            state: ActiveValueState::Unset,
        }
    }

    /// Check if the state of an [ActiveValue] is `ActiveValueState::Unset`
    /// which returns true if it is
    pub fn is_unset(&self) -> bool {
        matches!(self.state, ActiveValueState::Unset)
    }

    /// Get the mutable value of the `value` field of an [ActiveValue]
    /// also setting it's state to `ActiveValueState::Unset`
    pub fn take(&mut self) -> Option<V> {
        self.state = ActiveValueState::Unset;
        self.value.take()
    }

    /// Get an owned value of the `value` field of the [ActiveValue]
    pub fn unwrap(self) -> V {
        self.value.unwrap()
    }

    /// Check is a [Value] exists or not
    pub fn into_value(self) -> Option<Value> {
        self.value.map(Into::into)
    }

    /// Wrap the [Value] into a `ActiveValue<Value>`
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

impl<V> From<ActiveValue<V>> for ActiveValue<Option<V>>
where
    V: Into<Value> + Nullable,
{
    fn from(value: ActiveValue<V>) -> Self {
        match value.state {
            ActiveValueState::Set => Set(value.value),
            ActiveValueState::Unset => Unset(None),
            ActiveValueState::Unchanged => ActiveValue::unchanged(value.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::*;

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_into_active_model_1() {
        use crate::entity::*;

        mod my_fruit {
            pub use super::fruit::*;
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(DeriveIntoActiveModel)]
            pub struct NewFruit {
                // id is omitted
                pub name: String,
                // it is required as opposed to optional in Model
                pub cake_id: i32,
            }
        }

        assert_eq!(
            my_fruit::NewFruit {
                name: "Apple".to_owned(),
                cake_id: 1,
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: Unset(None),
                name: Set("Apple".to_owned()),
                cake_id: Set(Some(1)),
            }
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_derive_into_active_model_2() {
        use crate::entity::*;

        mod my_fruit {
            pub use super::fruit::*;
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(DeriveIntoActiveModel)]
            pub struct UpdateFruit {
                pub cake_id: Option<Option<i32>>,
            }
        }

        assert_eq!(
            my_fruit::UpdateFruit {
                cake_id: Some(Some(1)),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: Unset(None),
                name: Unset(None),
                cake_id: Set(Some(1)),
            }
        );

        assert_eq!(
            my_fruit::UpdateFruit {
                cake_id: Some(None),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: Unset(None),
                name: Unset(None),
                cake_id: Set(None),
            }
        );

        assert_eq!(
            my_fruit::UpdateFruit { cake_id: None }.into_active_model(),
            fruit::ActiveModel {
                id: Unset(None),
                name: Unset(None),
                cake_id: Unset(None),
            }
        );
    }
}
