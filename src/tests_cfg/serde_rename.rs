#![allow(clippy::unwrap_used)]
use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "user")]
#[cfg_attr(feature = "with-json", serde(rename_all = "camelCase"))]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub is_admin: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub mod field_rename {
    use crate as sea_orm;
    use sea_orm::entity::prelude::*;

    #[cfg(feature = "with-json")]
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
    #[sea_orm(table_name = "order")]
    #[cfg_attr(feature = "with-json", serde(rename_all = "camelCase"))]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub order_date: String,
        // #[serde(rename = "...")] applies to both serialize and deserialize
        #[cfg_attr(feature = "with-json", serde(rename = "order-id"))]
        pub order_id: String,
        // serialize only - does not affect json_key (uses camelCase)
        #[cfg_attr(feature = "with-json", serde(rename(serialize = "serializedOnly")))]
        pub ser_only: String,
        // deserialize only - affects json_key
        #[cfg_attr(feature = "with-json", serde(rename(deserialize = "deOnly")))]
        pub de_only: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod no_serde_rename {
    use crate as sea_orm;
    use sea_orm::entity::prelude::*;

    #[cfg(feature = "with-json")]
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
    #[sea_orm(table_name = "legacy")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub field_name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[cfg(all(test, feature = "with-json"))]
mod tests {
    use super::*;
    use crate::ActiveValue;
    use crate::entity::ActiveModelTrait;

    #[test]
    fn test_rename_all() {
        // json_key returns camelCase names
        assert_eq!(Column::Id.json_key(), "id");
        assert_eq!(Column::FirstName.json_key(), "firstName");
        assert_eq!(Column::LastName.json_key(), "lastName");
        assert_eq!(Column::Email.json_key(), "email");
        assert_eq!(Column::IsAdmin.json_key(), "isAdmin");

        // from_json uses camelCase keys
        let json = serde_json::json!({
            "id": 1,
            "firstName": "Max",
            "lastName": "Hermit",
            "email": "max@domain.com",
            "isAdmin": true,
        });

        let am = ActiveModel::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert_eq!(am.first_name, ActiveValue::Set("Max".to_string()));
        assert_eq!(am.last_name, ActiveValue::Set("Hermit".to_string()));
        assert_eq!(am.email, ActiveValue::Set("max@domain.com".to_string()));
        assert_eq!(am.is_admin, ActiveValue::Set(true));
    }

    #[test]
    fn test_field_rename() {
        use field_rename::{ActiveModel, Column};

        // json_key behavior:
        // - rename = "..." uses that name
        // - rename(deserialize = "...") uses deserialize name
        // - rename(serialize = "...") falls back to rename_all
        assert_eq!(Column::Id.json_key(), "id");
        assert_eq!(Column::OrderDate.json_key(), "orderDate");
        assert_eq!(Column::OrderId.json_key(), "order-id");
        assert_eq!(Column::SerOnly.json_key(), "serOnly"); // camelCase, not "serializedOnly"
        assert_eq!(Column::DeOnly.json_key(), "deOnly");

        // from_json uses deserialize names
        let json = serde_json::json!({
            "id": 1,
            "orderDate": "2024-01-01",
            "order-id": "ORD123",
            "serOnly": "ser-value",
            "deOnly": "de-value"
        });

        let am = ActiveModel::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert_eq!(am.order_date, ActiveValue::Set("2024-01-01".to_string()));
        assert_eq!(am.order_id, ActiveValue::Set("ORD123".to_string()));
        assert_eq!(am.ser_only, ActiveValue::Set("ser-value".to_string()));
        assert_eq!(am.de_only, ActiveValue::Set("de-value".to_string()));
    }

    #[test]
    fn test_no_serde_rename() {
        use no_serde_rename::{ActiveModel, Column};

        assert_eq!(Column::Id.json_key(), "id");
        assert_eq!(Column::FieldName.json_key(), "field_name");

        let json = serde_json::json!({
            "id": 1,
            "field_name": "value"
        });

        let am = ActiveModel::from_json(json).unwrap();

        assert_eq!(am.id, ActiveValue::Set(1));
        assert_eq!(am.field_name, ActiveValue::Set("value".to_string()));
    }
}
