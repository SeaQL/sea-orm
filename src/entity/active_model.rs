use crate::{
    error::*, ConnectionTrait, DeleteResult, EntityTrait, Iterable, PrimaryKeyToColumn, Value,
};
use async_trait::async_trait;
use sea_query::{Nullable, ValueTuple};
use std::fmt::Debug;

pub use ActiveValue::NotSet;

/// Defines a stateful value used in ActiveModel.
///
/// There are three possible state represented by three enum variants.
/// - [ActiveValue::Set]: A defined [Value] actively being set
/// - [ActiveValue::Unchanged]: A defined [Value] remain unchanged
/// - [ActiveValue::NotSet]: An undefined [Value]
///
/// The stateful value is useful when constructing UPDATE SQL statement,
/// see an example below.
///
/// # Examples
///
/// ```
/// use sea_orm::tests_cfg::{cake, fruit};
/// use sea_orm::{entity::*, query::*, DbBackend};
///
/// // The code snipped below does an UPDATE operation on a `ActiveValue`
/// assert_eq!(
///     Update::one(fruit::ActiveModel {
///         id: ActiveValue::set(1),
///         name: ActiveValue::set("Orange".to_owned()),
///         cake_id: ActiveValue::not_set(),
///     })
///     .build(DbBackend::Postgres)
///     .to_string(),
///     r#"UPDATE "fruit" SET "name" = 'Orange' WHERE "fruit"."id" = 1"#
/// );
/// ```
#[derive(Clone, Debug)]
pub enum ActiveValue<V>
where
    V: Into<Value>,
{
    /// A defined [Value] actively being set
    Set(V),
    /// A defined [Value] remain unchanged
    Unchanged(V),
    /// An undefined [Value]
    NotSet,
}

/// Defines a set operation on an [ActiveValue]
#[allow(non_snake_case)]
pub fn Set<V>(v: V) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::set(v)
}

/// Defines an not set operation on an [ActiveValue]
#[deprecated(
    since = "0.5.0",
    note = "Please use [`ActiveValue::NotSet`] or [`NotSet`]"
)]
#[allow(non_snake_case)]
pub fn Unset<V>(_: Option<bool>) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::not_set()
}

/// Defines an unchanged operation on an [ActiveValue]
#[allow(non_snake_case)]
pub fn Unchanged<V>(value: V) -> ActiveValue<V>
where
    V: Into<Value>,
{
    ActiveValue::unchanged(value)
}

/// A Trait for ActiveModel to perform Create, Update or Delete operation.
/// The type must also implement the [EntityTrait].
/// See module level docs [crate::entity] for a full example
#[async_trait]
pub trait ActiveModelTrait: Clone + Debug {
    /// The Entity this ActiveModel belongs to
    type Entity: EntityTrait;

    /// Get a mutable [ActiveValue] from an ActiveModel
    fn take(&mut self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    /// Get a immutable [ActiveValue] from an ActiveModel
    fn get(&self, c: <Self::Entity as EntityTrait>::Column) -> ActiveValue<Value>;

    /// Set the Value into an ActiveModel
    fn set(&mut self, c: <Self::Entity as EntityTrait>::Column, v: Value);

    /// Set the state of an [ActiveValue] to the not set state
    fn not_set(&mut self, c: <Self::Entity as EntityTrait>::Column);

    /// Check the state of a [ActiveValue]
    fn is_not_set(&self, c: <Self::Entity as EntityTrait>::Column) -> bool;

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
            4 => {
                let s1 = next!();
                let s2 = next!();
                let s3 = next!();
                let s4 = next!();
                Some(ValueTuple::Four(s1, s2, s3, s4))
            }
            5 => {
                let s1 = next!();
                let s2 = next!();
                let s3 = next!();
                let s4 = next!();
                let s5 = next!();
                Some(ValueTuple::Five(s1, s2, s3, s4, s5))
            }
            6 => {
                let s1 = next!();
                let s2 = next!();
                let s3 = next!();
                let s4 = next!();
                let s5 = next!();
                let s6 = next!();
                Some(ValueTuple::Six(s1, s2, s3, s4, s5, s6))
            }
            _ => panic!("The arity cannot be larger than 6"),
        }
    }

    /// Perform an `INSERT` operation on the ActiveModel
    ///
    /// # Example (Postgres)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![cake::Model {
    /// #             id: 15,
    /// #             name: "Apple Pie".to_owned(),
    /// #         }],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(
    ///     apple.insert(&db).await?,
    ///     cake::Model {
    ///         id: 15,
    ///         name: "Apple Pie".to_owned(),
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1) RETURNING "id", "name""#,
    ///         vec!["Apple Pie".into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (MySQL)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::MySql)
    /// #     .append_query_results(vec![
    /// #         vec![cake::Model {
    /// #             id: 15,
    /// #             name: "Apple Pie".to_owned(),
    /// #         }],
    /// #     ])
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 15,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(
    ///     apple.insert(&db).await?,
    ///     cake::Model {
    ///         id: 15,
    ///         name: "Apple Pie".to_owned(),
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"INSERT INTO `cake` (`name`) VALUES (?)"#,
    ///             vec!["Apple Pie".into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = ? LIMIT ?"#,
    ///             vec![15.into(), 1u64.into()]
    ///         )
    ///     ]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    async fn insert<'a, C>(self, db: &'a C) -> Result<<Self::Entity as EntityTrait>::Model, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let am = ActiveModelBehavior::before_save(self, true)?;
        let model = <Self::Entity as EntityTrait>::insert(am)
            .exec_with_returning(db)
            .await?;
        Self::after_save(model, true)
    }

    /// Perform the `UPDATE` operation on an ActiveModel
    ///
    /// # Example (Postgres)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 1,
    /// #             name: "Orange".to_owned(),
    /// #             cake_id: None,
    /// #         }],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(
    ///     orange.update(&db).await?,
    ///     fruit::Model {
    ///         id: 1,
    ///         name: "Orange".to_owned(),
    ///         cake_id: None,
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"UPDATE "fruit" SET "name" = $1 WHERE "fruit"."id" = $2 RETURNING "id", "name", "cake_id""#,
    ///         vec!["Orange".into(), 1i32.into()]
    ///     )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example (MySQL)
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::MySql)
    /// #     .append_query_results(vec![
    /// #         vec![fruit::Model {
    /// #             id: 1,
    /// #             name: "Orange".to_owned(),
    /// #             cake_id: None,
    /// #         }],
    /// #     ])
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// assert_eq!(
    ///     orange.update(&db).await?,
    ///     fruit::Model {
    ///         id: 1,
    ///         name: "Orange".to_owned(),
    ///         cake_id: None,
    ///     }
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"UPDATE `fruit` SET `name` = ? WHERE `fruit`.`id` = ?"#,
    ///             vec!["Orange".into(), 1i32.into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::MySql,
    ///             r#"SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = ? LIMIT ?"#,
    ///             vec![1i32.into(), 1u64.into()]
    ///         )]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    async fn update<'a, C>(self, db: &'a C) -> Result<<Self::Entity as EntityTrait>::Model, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let am = ActiveModelBehavior::before_save(self, false)?;
        let model: <Self::Entity as EntityTrait>::Model = Self::Entity::update(am).exec(db).await?;
        Self::after_save(model, false)
    }

    /// Insert the model if primary key is `NotSet`, update otherwise.
    /// Only works if the entity has auto increment primary key.
    async fn save<'a, C>(self, db: &'a C) -> Result<Self, DbErr>
    where
        <Self::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let mut is_update = true;
        for key in <Self::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            if self.is_not_set(col) {
                is_update = false;
                break;
            }
        }
        let res = if !is_update {
            self.insert(db).await
        } else {
            self.update(db).await
        }?;
        Ok(res.into_active_model())
    }

    /// Delete an active model by its primary key
    ///
    /// # Example
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(3),
    ///     ..Default::default()
    /// };
    ///
    /// let delete_result = orange.delete(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#,
    ///         vec![3i32.into()]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    async fn delete<'a, C>(self, db: &'a C) -> Result<DeleteResult, DbErr>
    where
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        let am = ActiveModelBehavior::before_delete(self)?;
        let am_clone = am.clone();
        let delete_res = Self::Entity::delete(am).exec(db).await?;
        ActiveModelBehavior::after_delete(am_clone)?;
        Ok(delete_res)
    }
}

/// A Trait for overriding the ActiveModel behavior
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
    fn after_save(
        model: <Self::Entity as EntityTrait>::Model,
        insert: bool,
    ) -> Result<<Self::Entity as EntityTrait>::Model, DbErr> {
        Ok(model)
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

/// A Trait for any type that can be converted into an ActiveModel
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
                    None => NotSet,
                }
            }
        }

        impl IntoActiveValue<Option<$ty>> for Option<Option<$ty>> {
            fn into_active_value(self) -> ActiveValue<Option<$ty>> {
                match self {
                    Some(value) => Set(value),
                    None => NotSet,
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

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTimeUtc, Set);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTimeLocal, Set);

#[cfg(feature = "with-rust_decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-rust_decimal")))]
impl_into_active_value!(crate::prelude::Decimal, Set);

#[cfg(feature = "with-uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-uuid")))]
impl_into_active_value!(crate::prelude::Uuid, Set);

impl<V> Default for ActiveValue<V>
where
    V: Into<Value>,
{
    /// Create an [ActiveValue::NotSet]
    fn default() -> Self {
        Self::NotSet
    }
}

impl<V> ActiveValue<V>
where
    V: Into<Value>,
{
    /// Create an [ActiveValue::Set]
    pub fn set(value: V) -> Self {
        Self::Set(value)
    }

    /// Check if the [ActiveValue] is [ActiveValue::Set]
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set(_))
    }

    /// Create an [ActiveValue::Unchanged]
    pub fn unchanged(value: V) -> Self {
        Self::Unchanged(value)
    }

    /// Check if the [ActiveValue] is [ActiveValue::Unchanged]
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Self::Unchanged(_))
    }

    /// Create an [ActiveValue::NotSet]
    pub fn not_set() -> Self {
        Self::default()
    }

    /// Check if the [ActiveValue] is [ActiveValue::NotSet]
    pub fn is_not_set(&self) -> bool {
        matches!(self, Self::NotSet)
    }

    /// Get the mutable value an [ActiveValue]
    /// also setting itself to [ActiveValue::NotSet]
    pub fn take(&mut self) -> Option<V> {
        match std::mem::take(self) {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => Some(value),
            ActiveValue::NotSet => None,
        }
    }

    /// Get an owned value of the [ActiveValue]
    pub fn unwrap(self) -> V {
        match self {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => value,
            ActiveValue::NotSet => panic!("Cannot unwrap ActiveValue::NotSet"),
        }
    }

    /// Check is a [Value] exists or not
    pub fn into_value(self) -> Option<Value> {
        match self {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => Some(value.into()),
            ActiveValue::NotSet => None,
        }
    }

    /// Wrap the [Value] into a `ActiveValue<Value>`
    pub fn into_wrapped_value(self) -> ActiveValue<Value> {
        match self {
            Self::Set(value) => ActiveValue::set(value.into()),
            Self::Unchanged(value) => ActiveValue::unchanged(value.into()),
            Self::NotSet => ActiveValue::not_set(),
        }
    }
}

impl<V> std::convert::AsRef<V> for ActiveValue<V>
where
    V: Into<Value>,
{
    fn as_ref(&self) -> &V {
        match self {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => value,
            ActiveValue::NotSet => panic!("Cannot borrow ActiveValue::NotSet"),
        }
    }
}

impl<V> PartialEq for ActiveValue<V>
where
    V: Into<Value> + std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ActiveValue::Set(l), ActiveValue::Set(r)) => l == r,
            (ActiveValue::Unchanged(l), ActiveValue::Unchanged(r)) => l == r,
            (ActiveValue::NotSet, ActiveValue::NotSet) => true,
            _ => false,
        }
    }
}

impl<V> From<ActiveValue<V>> for ActiveValue<Option<V>>
where
    V: Into<Value> + Nullable,
{
    fn from(value: ActiveValue<V>) -> Self {
        match value {
            ActiveValue::Set(value) => ActiveValue::set(Some(value)),
            ActiveValue::Unchanged(value) => ActiveValue::unchanged(Some(value)),
            ActiveValue::NotSet => ActiveValue::not_set(),
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
                id: NotSet,
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
                id: NotSet,
                name: NotSet,
                cake_id: Set(Some(1)),
            }
        );

        assert_eq!(
            my_fruit::UpdateFruit {
                cake_id: Some(None),
            }
            .into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: NotSet,
                cake_id: Set(None),
            }
        );

        assert_eq!(
            my_fruit::UpdateFruit { cake_id: None }.into_active_model(),
            fruit::ActiveModel {
                id: NotSet,
                name: NotSet,
                cake_id: NotSet,
            }
        );
    }
}
