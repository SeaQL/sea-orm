use crate::Value;
use sea_query::Nullable;
use std::fmt::Debug;

pub use ActiveValue::{NotSet, Set, Unchanged};

/// The state of a field in an [ActiveModel][ActiveModelTrait].
///
/// There are three possible states represented by three enum variants:
///
/// - [Set] - a value that's explicitly set by the application and sent to the database.
/// - [Unchanged] - an existing, unchanged value from the database.
/// - [NotSet] - an undefined value (nothing is sent to the database).
///
/// The difference between these states is useful
/// when constructing `INSERT` and `UPDATE` SQL statements (see an example below).
/// It's also useful for knowing which fields have changed in a record.
///
/// # Examples
///
/// ```
/// use sea_orm::tests_cfg::{cake, fruit};
/// use sea_orm::{DbBackend, entity::*, query::*};
///
/// // Here, we use `NotSet` to let the database automatically generate an `id`.
/// // This is different from `Set(None)` that explicitly sets `cake_id` to `NULL`.
/// assert_eq!(
///     Insert::one(fruit::ActiveModel {
///         id: ActiveValue::NotSet,
///         name: ActiveValue::Set("Orange".to_owned()),
///         cake_id: ActiveValue::Set(None),
///     })
///     .build(DbBackend::Postgres)
///     .to_string(),
///     r#"INSERT INTO "fruit" ("name", "cake_id") VALUES ('Orange', NULL)"#
/// );
///
/// // Here, we update the record, set `cake_id` to the new value
/// // and use `NotSet` to avoid updating the `name` field.
/// // `id` is the primary key, so it's used in the condition and not updated.
/// assert_eq!(
///     Update::one(fruit::ActiveModel {
///         id: ActiveValue::Unchanged(1),
///         name: ActiveValue::NotSet,
///         cake_id: ActiveValue::Set(Some(2)),
///     })
///     .validate()
///     .unwrap()
///     .build(DbBackend::Postgres)
///     .to_string(),
///     r#"UPDATE "fruit" SET "cake_id" = 2 WHERE "fruit"."id" = 1"#
/// );
/// ```
#[derive(Clone, Debug)]
pub enum ActiveValue<V>
where
    V: Into<Value>,
{
    /// A [Value] that's explicitly set by the application and sent to the database.
    ///
    /// Use this to insert or set a specific value.
    ///
    /// When editing an existing value, you can use [set_if_not_equals][ActiveValue::set_if_not_equals]
    /// to preserve the [Unchanged] state when the new value is the same as the old one.
    /// Then you can meaningfully use methods like [ActiveModelTrait::is_changed].
    Set(V),
    /// An existing, unchanged [Value] from the database.
    ///
    /// You get these when you query an existing [Model][crate::ModelTrait]
    /// from the database and convert it into an [ActiveModel][ActiveModelTrait].
    ///
    /// When you edit it, you can use [set_if_not_equals][ActiveValue::set_if_not_equals]
    /// to preserve this "unchanged" state if the new value is the same as the old one.
    /// Then you can meaningfully use methods like [ActiveModelTrait::is_changed].
    Unchanged(V),
    /// An undefined [Value]. Nothing is sent to the database.
    ///
    /// When you create a new [ActiveModel][ActiveModelTrait],
    /// its fields are [NotSet][ActiveValue::NotSet] by default.
    ///
    /// This can be useful when:
    ///
    /// - You insert a new record and want the database to generate a default value (e.g., an id).
    /// - In an `UPDATE` statement, you don't want to update some field.
    NotSet,
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

/// Any type that can be converted into an [ActiveValue]
pub trait IntoActiveValue<V>
where
    V: Into<Value>,
{
    /// Method to perform the conversion
    fn into_active_value(self) -> ActiveValue<V>;
}

impl<V> IntoActiveValue<V> for Option<V>
where
    V: IntoActiveValue<V> + Into<Value> + Nullable,
{
    fn into_active_value(self) -> ActiveValue<V> {
        match self {
            Some(value) => Set(value),
            None => NotSet,
        }
    }
}

impl<V> IntoActiveValue<Option<V>> for Option<Option<V>>
where
    V: IntoActiveValue<V> + Into<Value> + Nullable,
{
    fn into_active_value(self) -> ActiveValue<Option<V>> {
        match self {
            Some(value) => Set(value),
            None => NotSet,
        }
    }
}

macro_rules! impl_into_active_value {
    ($ty: ty) => {
        impl IntoActiveValue<$ty> for $ty {
            fn into_active_value(self) -> ActiveValue<$ty> {
                Set(self)
            }
        }
    };
}

impl_into_active_value!(bool);
impl_into_active_value!(i8);
impl_into_active_value!(i16);
impl_into_active_value!(i32);
impl_into_active_value!(i64);
impl_into_active_value!(u8);
impl_into_active_value!(u16);
impl_into_active_value!(u32);
impl_into_active_value!(u64);
impl_into_active_value!(f32);
impl_into_active_value!(f64);
impl_into_active_value!(&'static str);
impl_into_active_value!(String);
impl_into_active_value!(Vec<u8>);

#[cfg(feature = "with-json")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-json")))]
impl_into_active_value!(crate::prelude::Json);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::Date);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::Time);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTime);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTimeWithTimeZone);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTimeUtc);

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
impl_into_active_value!(crate::prelude::DateTimeLocal);

#[cfg(feature = "with-rust_decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-rust_decimal")))]
impl_into_active_value!(crate::prelude::Decimal);

#[cfg(feature = "with-bigdecimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-bigdecimal")))]
impl_into_active_value!(crate::prelude::BigDecimal);

#[cfg(feature = "with-uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-uuid")))]
impl_into_active_value!(crate::prelude::Uuid);

#[cfg(feature = "with-time")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-time")))]
impl_into_active_value!(crate::prelude::TimeDate);

#[cfg(feature = "with-time")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-time")))]
impl_into_active_value!(crate::prelude::TimeTime);

#[cfg(feature = "with-time")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-time")))]
impl_into_active_value!(crate::prelude::TimeDateTime);

#[cfg(feature = "with-time")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-time")))]
impl_into_active_value!(crate::prelude::TimeDateTimeWithTimeZone);

#[cfg(feature = "with-ipnetwork")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-ipnetwork")))]
impl_into_active_value!(crate::prelude::IpNetwork);

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

    /// Take ownership of the inner value, also setting self to `NotSet`
    pub fn take(&mut self) -> Option<V> {
        match std::mem::take(self) {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => Some(value),
            ActiveValue::NotSet => None,
        }
    }

    /// Get an owned value of the [ActiveValue]
    ///
    /// # Panics
    ///
    /// Panics if it is [ActiveValue::NotSet]
    pub fn unwrap(self) -> V {
        match self {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => value,
            ActiveValue::NotSet => panic!("Cannot unwrap ActiveValue::NotSet"),
        }
    }

    /// Take ownership of the inner value, consuming self
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

    /// Reset the value from [ActiveValue::Unchanged] to [ActiveValue::Set],
    /// leaving [ActiveValue::NotSet] untouched.
    pub fn reset(&mut self) {
        *self = match self.take() {
            Some(value) => ActiveValue::Set(value),
            None => ActiveValue::NotSet,
        };
    }

    /// `Set(value)`, except when [`self.is_unchanged()`][ActiveValue#method.is_unchanged]
    /// and `value` equals the current [Unchanged][ActiveValue::Unchanged] value.
    ///
    /// This is useful when you have an [Unchanged][ActiveValue::Unchanged] value from the database,
    /// then update it using this method,
    /// and then use [`.is_unchanged()`][ActiveValue#method.is_unchanged] to see whether it has *actually* changed.
    ///
    /// The same nice effect applies to the entire `ActiveModel`.
    /// You can now meaningfully use [ActiveModelTrait::is_changed][ActiveModelTrait#method.is_changed]
    /// to see whether are any changes that need to be saved to the database.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use sea_orm::ActiveValue;
    /// #
    /// let mut value = ActiveValue::Unchanged("old");
    ///
    /// // This wouldn't be the case if we used plain `value = Set("old");`
    /// value.set_if_not_equals("old");
    /// assert!(value.is_unchanged());
    ///
    /// // Only when we change the actual `&str` value, it becomes `Set`
    /// value.set_if_not_equals("new");
    /// assert_eq!(value.is_unchanged(), false);
    /// assert_eq!(value, ActiveValue::Set("new"));
    /// ```
    pub fn set_if_not_equals(&mut self, value: V)
    where
        V: PartialEq,
    {
        match self {
            ActiveValue::Unchanged(current) if &value == current => {}
            _ => *self = ActiveValue::Set(value),
        }
    }

    /// `Set(value)`, except when [`self.is_unchanged()`][ActiveValue#method.is_unchanged],
    /// `value` equals the current [Unchanged][ActiveValue::Unchanged] value, and `value`
    /// does not match a given predicate.
    ///
    /// This is useful in the same situations as [ActiveValue#method.set_if_not_equals] as
    /// well as when you want to leave an existing [Set][ActiveValue::Set] value alone
    /// depending on a condition, such as ensuring a `None` value never replaced an
    /// existing `Some` value. This can come up when trying to merge two [ActiveValue]s.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use sea_orm::ActiveValue;
    /// #
    /// let mut value = ActiveValue::Set(Some("old"));
    ///
    /// // since Option::is_some(None) == false, we leave the existing set value alone
    /// value.set_if_not_equals_and(None, Option::is_some);
    /// assert_eq!(value, ActiveValue::Set(Some("old")));
    ///
    /// // since Option::is_some(Some("new")) == true, we replace the set value
    /// value.set_if_not_equals_and(Some("new"), Option::is_some);
    /// assert_eq!(value, ActiveValue::Set(Some("new")));
    /// ```
    pub fn set_if_not_equals_and(&mut self, value: V, f: impl FnOnce(&V) -> bool)
    where
        V: PartialEq,
    {
        match self {
            ActiveValue::Unchanged(current) if &value == current => {}
            ActiveValue::Set(_) if !f(&value) => {}
            _ => *self = ActiveValue::Set(value),
        }
    }

    /// Get the inner value, unless `self` is [NotSet][ActiveValue::NotSet].
    ///
    /// There's also a panicking version: [ActiveValue::as_ref].
    ///
    /// ## Examples
    ///
    /// ```
    /// # use sea_orm::ActiveValue;
    /// #
    /// assert_eq!(ActiveValue::Unchanged(42).try_as_ref(), Some(&42));
    /// assert_eq!(ActiveValue::Set(42).try_as_ref(), Some(&42));
    /// assert_eq!(ActiveValue::NotSet.try_as_ref(), None::<&i32>);
    /// ```
    pub fn try_as_ref(&self) -> Option<&V> {
        match self {
            ActiveValue::Set(value) | ActiveValue::Unchanged(value) => Some(value),
            ActiveValue::NotSet => None,
        }
    }
}

impl<V> std::convert::AsRef<V> for ActiveValue<V>
where
    V: Into<Value>,
{
    /// # Panics
    ///
    /// Panics if it is [ActiveValue::NotSet].
    ///
    /// See [ActiveValue::try_as_ref] for a fallible non-panicking version.
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
