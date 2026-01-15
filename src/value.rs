use crate::sea_query::{Nullable, ValueType};
use crate::{ActiveValue, Value};

mod timestamp;
use timestamp::*;

#[cfg(feature = "with-chrono")]
mod with_chrono;
#[cfg(feature = "with-chrono")]
pub use with_chrono::*;

#[cfg(feature = "with-time")]
mod with_time;
#[cfg(feature = "with-time")]
pub use with_time::*;

#[cfg(feature = "with-uuid")]
mod text_uuid;
#[cfg(feature = "with-uuid")]
pub use text_uuid::*;

/// Default value for T
pub trait DefaultActiveValue {
    /// `Default::default()` if implemented, dummy value otherwise
    fn default_value(&self) -> Self;
}

/// Default value for Option<T>
pub trait DefaultActiveValueNone {
    /// Always `None`
    fn default_value(&self) -> Self;
}

/// Default value for types that's not nullable
pub trait DefaultActiveValueNotSet {
    /// The owned value type
    type Value;

    /// Always `NotSet`
    fn default_value(&self) -> Self::Value;
}

impl<V> DefaultActiveValue for ActiveValue<V>
where
    V: Into<Value> + ValueType + Nullable,
{
    fn default_value(&self) -> Self {
        match V::try_from(V::null().dummy_value()) {
            Ok(v) => ActiveValue::Set(v),
            Err(_) => ActiveValue::NotSet,
        }
    }
}

impl<V> DefaultActiveValueNone for ActiveValue<Option<V>>
where
    V: Into<Value> + Nullable,
{
    fn default_value(&self) -> Self {
        ActiveValue::Set(None)
    }
}

impl<V> DefaultActiveValueNotSet for &ActiveValue<V>
where
    V: Into<Value>,
{
    type Value = ActiveValue<V>;

    fn default_value(&self) -> Self::Value {
        ActiveValue::NotSet
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::TimeDateTime;

    #[test]
    fn test_default_value() {
        let v = (&ActiveValue::<i32>::NotSet).default_value();
        assert_eq!(v, ActiveValue::Set(0));

        let v = (&ActiveValue::<Option<i32>>::NotSet).default_value();
        assert_eq!(v, ActiveValue::Set(None));

        let v = (&ActiveValue::<String>::NotSet).default_value();
        assert_eq!(v, ActiveValue::Set("".to_owned()));

        let v = (&ActiveValue::<Option<String>>::NotSet).default_value();
        assert_eq!(v, ActiveValue::Set(None));

        let v = (&ActiveValue::<TimeDateTime>::NotSet).default_value();
        assert!(matches!(v, ActiveValue::Set(_)));
    }
}
