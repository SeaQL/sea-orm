use crate::sea_query::{ArrayType, ColumnType, Nullable, ValueType, ValueTypeErr};
use crate::{ColIdx, QueryResult, TryGetError, TryGetable, Value, error::json_err};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Wrap a Rust value that should be encoded/decoded via a JSON/JSONB column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonField<T>(pub T);

impl<T> From<T> for JsonField<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Serialize for JsonField<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for JsonField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(JsonField)
    }
}

impl<T> From<JsonField<T>> for Value
where
    T: Serialize,
{
    fn from(source: JsonField<T>) -> Self {
        Value::Json(Some(serde_json::to_value(source.0).expect(concat!(
            "Failed to serialize JSON value for '",
            stringify!(JsonField),
            "'"
        ))))
    }
}

impl<T> ValueType for JsonField<T>
where
    for<'de> T: Deserialize<'de>,
{
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            // TODO: Should we convert None to Null?
            Value::Json(Some(json)) => serde_json::from_value(json)
                .map(JsonField)
                .map_err(|_| ValueTypeErr),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        format!("JsonField<{}>", std::any::type_name::<T>())
    }

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> ColumnType {
        ColumnType::Json
    }
}

impl<T> Nullable for JsonField<T> {
    fn null() -> Value {
        Value::Json(None)
    }
}

impl<T> TryGetable for JsonField<T>
where
    for<'de> T: Deserialize<'de>,
{
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let json = serde_json::Value::try_get_by(res, index)?;
        serde_json::from_value(json)
            .map(JsonField)
            .map_err(|e| TryGetError::DbErr(json_err(e)))
    }
}
