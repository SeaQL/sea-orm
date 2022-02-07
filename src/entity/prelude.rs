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

#[cfg(feature = "with-time")]
pub use time::Date;

#[cfg(feature = "with-time")]
pub use time::Time;

#[cfg(feature = "with-time")]
pub use time::PrimitiveDateTime as DateTime;

/// Handles the time and dates
#[cfg(feature = "with-time")]
pub type DateTimeWithTimeZone = time::OffsetDateTime;

#[cfg(feature = "with-rust_decimal")]
pub use rust_decimal::Decimal;

#[cfg(feature = "with-uuid")]
pub use uuid::Uuid;
