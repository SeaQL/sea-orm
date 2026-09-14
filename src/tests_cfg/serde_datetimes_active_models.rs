/// Regression test for [https://github.com/SeaQL/sea-orm/issues/3175]
/// `ActiveModel::from_json` failed to deserialize time columns to `NotSet`
/// when the field was missing from the JSON payload, because the
/// trait-default implementation round-tripped through the model after
/// merging SQL-literal dummy values. 

#[cfg(feature = "with-time")]
mod time_model {
    use crate as sea_orm;
    use sea_orm::entity::prelude::*;
    use serde::{Serialize, Deserialize};

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
    #[sea_orm(table_name = "time")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i32,
        #[serde(with = "time::serde::timestamp")]
        pub created_at: TimeDateTimeWithTimeZone,
    }
    
    impl ActiveModelBehavior for ActiveModel {}
}

#[cfg(feature = "with-chrono")]
mod chrono_model {
    use crate as sea_orm;
    use sea_orm::entity::prelude::*;
    use serde::{Serialize, Deserialize};

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, DeriveEntityModel)]
    #[sea_orm(table_name = "time")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: i32,
        #[serde(rename = "tstamp", with = "chrono::serde::ts_seconds")]
        pub created_at: ChronoDateTimeUtc,
    }
    
    impl ActiveModelBehavior for ActiveModel {}
}

mod test {
    use super::{time_model::ActiveModel as TimeAM, chrono_model::ActiveModel as ChronoAM};
    use crate::{ActiveValue, entity::ActiveModelTrait};

    #[test]
    #[cfg(feature = "with-time")]
    fn test_from_json_missing_time_field_is_not_set() {
        let json = serde_json::json!({
            "id": 1,
        });

        let am = TimeAM::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert_eq!(am.created_at, ActiveValue::NotSet);
    }

    #[test]
    #[cfg(feature = "with-time")]
    fn test_from_json_present_time_field_is_set() {
        let json = serde_json::json!({
            "id": 1,
            "created_at": 1704067200,
        });

        let am = TimeAM::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert!(matches!(am.created_at, ActiveValue::Set(_)));
    }

    #[test]
    #[cfg(feature = "with-chrono")]
    fn test_from_json_missing_chrono_field_is_not_set() {
        let json = serde_json::json!({
            "id": 1,
        });

        let am = ChronoAM::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert_eq!(am.created_at, ActiveValue::NotSet);
    }

    #[test]
    #[cfg(feature = "with-chrono")]
    fn test_from_json_present_chrono_field_is_set() {
        let json = serde_json::json!({
            "id": 1,
            "tstamp": 1704067200,
        });

        let am = ChronoAM::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert!(matches!(am.created_at, ActiveValue::Set(_)));
    }
}
