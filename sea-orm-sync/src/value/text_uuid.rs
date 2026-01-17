use std::ops::{Deref, DerefMut};

use sea_query::{ValueType, ValueTypeErr};

use crate::TryGetable;
use crate::{self as sea_orm, TryFromU64};
use crate::{DbErr, TryGetError};

/// Newtype making sure that UUIDs will be stored as `TEXT` columns,
/// instead of `BLOB` (which is the default).
/// Advantages:
/// - TEXT makes it easier to interact with the SQLite DB directly
/// - Allows for queries like `WHERE id IN (<uuid>, <uuid>, ...)` which are
///   impossible to write with `BLOB` values
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct TextUuid(pub uuid::Uuid);

impl From<TextUuid> for sea_query::Value {
    fn from(value: TextUuid) -> Self {
        value.0.to_string().into()
    }
}

impl TryGetable for TextUuid {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let uuid_str: String = res.try_get_by(index)?;
        let uuid = uuid::Uuid::parse_str(&uuid_str).map_err(|e| {
            TryGetError::DbErr(DbErr::Type(format!("Failed to parse string as UUID: {e}")))
        })?;
        Ok(TextUuid(uuid))
    }
}

impl ValueType for TextUuid {
    fn try_from(v: sea_orm::Value) -> Result<Self, ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(s)) => {
                let uuid = uuid::Uuid::parse_str(&s).map_err(|_| ValueTypeErr)?;
                Ok(TextUuid(uuid))
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "TextUuid".to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        <String as sea_query::ValueType>::array_type()
    }

    fn column_type() -> sea_orm::ColumnType {
        <String as sea_query::ValueType>::column_type()
    }
}

// This seems to be required when using TextUuid as a primary key
impl TryFromU64 for TextUuid {
    fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::ConvertFromU64("TextUuid"))
    }
}

impl sea_query::Nullable for TextUuid {
    fn null() -> sea_orm::Value {
        <String as sea_query::Nullable>::null()
    }
}

impl sea_orm::IntoActiveValue<TextUuid> for TextUuid {
    fn into_active_value(self) -> crate::ActiveValue<TextUuid> {
        sea_orm::ActiveValue::Set(self)
    }
}

impl Deref for TextUuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl DerefMut for TextUuid {
    fn deref_mut(&mut self) -> &mut uuid::Uuid {
        &mut self.0
    }
}

impl From<uuid::Uuid> for TextUuid {
    fn from(value: uuid::Uuid) -> Self {
        TextUuid(value)
    }
}

impl From<TextUuid> for uuid::Uuid {
    fn from(value: TextUuid) -> Self {
        value.0
    }
}
