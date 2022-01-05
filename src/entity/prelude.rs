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

/// Handles the time and dates
#[cfg(feature = "with-chrono")]
pub type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;

/// Handles the time and dates in UTC
///
/// ### Example Usage
/// ```ignore
/// use chrono::{DateTime, NaiveDateTime, Utc};
/// use sea_orm::prelude::*;
///
/// let my_model = fruit::Model {
///        id: 3_i32,
///        name: "Fruit".to_owned(),
///        cake_id: Some(4),
///        timer: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
/// };
///
/// assert_eq!(
///         fruit::Model {
///             id: 3,
///             name: "Fruit".to_owned(),
///             cake_id: Some(4,),
///             timer: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
///         },
///         my_model
///     );
///
/// // Define a `Model` containing a type of `DateTimeUtc` field
/// #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
/// pub struct Model {
///     pub id: i32,
///     pub name: String,
///     pub cake_id: Option<i32>,
///     pub timer: DateTimeUtc,
/// }
///
/// #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
/// pub enum Column {
///     Id,
///     Name,
///     CakeId,
///     Timer,
/// }
/// ```
#[cfg(feature = "with-chrono")]
pub type DateTimeUtc = chrono::DateTime<chrono::Utc>;

#[cfg(feature = "with-rust_decimal")]
pub use rust_decimal::Decimal;

#[cfg(feature = "with-uuid")]
pub use uuid::Uuid;
