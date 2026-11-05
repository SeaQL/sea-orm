use std::{
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use sea_query::{ValueType, ValueTypeErr};

use crate::TryGetable;
use crate::{self as sea_orm, TryFromU64};

/// PostgreSQL `tsvector` type for full-text search.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct TsVector(pub String);

impl fmt::Display for TsVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for TsVector {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TsVector(s.to_string()))
    }
}

super::impl_serde_with_str!(TsVector);

impl From<TsVector> for sea_query::Value {
    fn from(value: TsVector) -> Self {
        value.0.into()
    }
}

impl From<String> for TsVector {
    fn from(s: String) -> Self {
        TsVector(s)
    }
}

impl From<&str> for TsVector {
    fn from(s: &str) -> Self {
        TsVector(s.to_owned())
    }
}

impl From<TsVector> for String {
    fn from(value: TsVector) -> Self {
        value.0
    }
}

impl TryGetable for TsVector {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let s: String = res.try_get_by(index)?;
        Ok(TsVector(s))
    }

    fn try_get_by_optional<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Option<Self>, sea_orm::TryGetError> {
        Ok(<String as TryGetable>::try_get_by_optional(res, index)?.map(TsVector))
    }
}

impl ValueType for TsVector {
    fn try_from(v: sea_orm::Value) -> Result<Self, ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(s)) => Ok(TsVector(*s)),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "TsVector".to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        <String as sea_query::ValueType>::array_type()
    }

    fn column_type() -> sea_orm::ColumnType {
        sea_orm::ColumnType::custom("tsvector")
    }
}

impl TryFromU64 for TsVector {
    fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::ConvertFromU64("TsVector"))
    }
}

impl sea_query::Nullable for TsVector {
    fn null() -> sea_orm::Value {
        <String as sea_query::Nullable>::null()
    }
}

impl sea_orm::IntoActiveValue<TsVector> for TsVector {
    fn into_active_value(self) -> crate::ActiveValue<TsVector> {
        sea_orm::ActiveValue::Set(self)
    }
}

impl Deref for TsVector {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for TsVector {
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

/// PostgreSQL `tsquery` type for full-text search.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct TsQuery(pub String);

impl fmt::Display for TsQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for TsQuery {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TsQuery(s.to_string()))
    }
}

super::impl_serde_with_str!(TsQuery);

impl From<TsQuery> for sea_query::Value {
    fn from(value: TsQuery) -> Self {
        value.0.into()
    }
}

impl From<String> for TsQuery {
    fn from(s: String) -> Self {
        TsQuery(s)
    }
}

impl From<&str> for TsQuery {
    fn from(s: &str) -> Self {
        TsQuery(s.to_owned())
    }
}

impl From<TsQuery> for String {
    fn from(value: TsQuery) -> Self {
        value.0
    }
}

impl TryGetable for TsQuery {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let s: String = res.try_get_by(index)?;
        Ok(TsQuery(s))
    }

    fn try_get_by_optional<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Option<Self>, sea_orm::TryGetError> {
        Ok(<String as TryGetable>::try_get_by_optional(res, index)?.map(TsQuery))
    }
}

impl ValueType for TsQuery {
    fn try_from(v: sea_orm::Value) -> Result<Self, ValueTypeErr> {
        match v {
            sea_orm::Value::String(Some(s)) => Ok(TsQuery(*s)),
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "TsQuery".to_string()
    }

    fn array_type() -> sea_query::ArrayType {
        <String as sea_query::ValueType>::array_type()
    }

    fn column_type() -> sea_orm::ColumnType {
        sea_orm::ColumnType::custom("tsquery")
    }
}

impl TryFromU64 for TsQuery {
    fn try_from_u64(_n: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::ConvertFromU64("TsQuery"))
    }
}

impl sea_query::Nullable for TsQuery {
    fn null() -> sea_orm::Value {
        <String as sea_query::Nullable>::null()
    }
}

impl sea_orm::IntoActiveValue<TsQuery> for TsQuery {
    fn into_active_value(self) -> crate::ActiveValue<TsQuery> {
        sea_orm::ActiveValue::Set(self)
    }
}

impl Deref for TsQuery {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl DerefMut for TsQuery {
    fn deref_mut(&mut self) -> &mut str {
        &mut self.0
    }
}
