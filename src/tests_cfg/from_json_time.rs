#![allow(clippy::unwrap_used)]
use crate as sea_orm;
use sea_orm::entity::prelude::*;

// Regression test for https://github.com/SeaQL/sea-orm/issues/3175
// `ActiveModel::from_json` failed to deserialize time columns to `NotSet`
// when the field was missing from the JSON payload, because the
// trait-default implementation round-tripped through the model after
// merging SQL-literal dummy values.
#[cfg(all(feature = "with-json", feature = "with-time"))]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(serde::Serialize, serde::Deserialize))]
#[sea_orm(table_name = "time_payload")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    #[cfg_attr(feature = "with-json", serde(with = "time::serde::timestamp"))]
    pub created_at: TimeDateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(all(test, feature = "with-json", feature = "with-time"))]
mod tests {
    use super::*;
    use crate::{ActiveValue, entity::ActiveModelTrait};

    #[test]
    fn test_from_json_missing_time_field_is_not_set() {
        let json = serde_json::json!({
            "id": 1,
        });

        let am = ActiveModel::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert_eq!(am.created_at, ActiveValue::NotSet);
    }

    #[test]
    fn test_from_json_present_time_field_is_set() {
        let json = serde_json::json!({
            "id": 1,
            "created_at": 1704067200,
        });

        let am = ActiveModel::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert!(matches!(am.created_at, ActiveValue::Set(_)));
    }
}
