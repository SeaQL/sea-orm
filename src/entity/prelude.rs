pub use crate::{
    error::*, ActiveEnum, ActiveModelBehavior, ActiveModelTrait, ColumnDef, ColumnTrait,
    ColumnType, DatabaseConnection, DbConn, EntityName, EntityTrait, EnumIter, ForeignKeyAction,
    Iden, IdenStatic, Linked, ModelTrait, PaginatorTrait, PrimaryKeyToColumn, PrimaryKeyTrait,
    QueryFilter, QueryResult, Related, RelationDef, RelationTrait, Select, Value,
};

#[cfg(feature = "macros")]
pub use crate::{
    DeriveActiveEnum, DeriveActiveModel, DeriveActiveModelBehavior, DeriveColumn,
    DeriveCustomColumn, DeriveEntity, DeriveEntityModel, DeriveIntoActiveModel, DeriveModel,
    DerivePrimaryKey, DeriveRelation,
};

#[cfg(feature = "with-json")]
pub use serde_json::Value as Json;

#[cfg(feature = "with-json")]
pub use sea_query::JsonValue;

#[cfg(feature = "with-chrono")]
pub use chrono::NaiveDate as Date;

#[cfg(feature = "with-chrono")]
pub use chrono::NaiveTime as Time;

#[cfg(feature = "with-chrono")]
pub use chrono::NaiveDateTime as DateTime;

/// Date time with fixed offset
#[cfg(feature = "with-chrono")]
pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;

/// Date time represented in UTC
#[cfg(feature = "with-chrono")]
pub type DateTimeUtc = chrono::DateTime<chrono::Utc>;

/// Date time represented in local time
#[cfg(feature = "with-chrono")]
pub type DateTimeLocal = chrono::DateTime<chrono::Local>;

#[cfg(feature = "with-chrono")]
pub use chrono::NaiveDate as ChronoDate;

#[cfg(feature = "with-chrono")]
pub use chrono::NaiveTime as ChronoTime;

#[cfg(feature = "with-chrono")]
pub use chrono::NaiveDateTime as ChronoDateTime;

/// Date time with fixed offset
#[cfg(feature = "with-chrono")]
pub type ChronoDateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;

/// Date time represented in UTC
#[cfg(feature = "with-chrono")]
pub type ChronoDateTimeUtc = chrono::DateTime<chrono::Utc>;

/// Date time represented in local time
#[cfg(feature = "with-chrono")]
pub type ChronoDateTimeLocal = chrono::DateTime<chrono::Local>;

#[cfg(feature = "with-time")]
pub use time::Date as TimeDate;

#[cfg(feature = "with-time")]
pub use time::Time as TimeTime;

#[cfg(feature = "with-time")]
pub use time::PrimitiveDateTime as TimeDateTime;

#[cfg(feature = "with-time")]
pub use time::OffsetDateTime as TimeDateTimeWithTimeZone;

#[cfg(feature = "with-rust_decimal")]
pub use rust_decimal::Decimal;

#[cfg(feature = "with-uuid")]
pub use uuid::Uuid;
