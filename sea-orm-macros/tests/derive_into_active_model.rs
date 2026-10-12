use sea_orm::{DeriveIntoActiveModel, IntoActiveModel, NotSet, Set};

mod fruit {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "fruit")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub cake_id: Option<i32>,
        pub default_value: i32,
        pub default_expr: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn test() {
    #[derive(DeriveIntoActiveModel)]
    #[sea_orm(
        table_name = "fruit",
        active_model = "fruit::ActiveModel",
        exhaustive,
        set(cake_id = "Some(42)")
    )]
    struct NewFruit {
        #[sea_orm(rename = "id", column_name = "fruit_id")]
        fruit_id: i32,
        #[sea_orm(rename = "name")]
        fruit_name: Option<String>,
        #[sea_orm(default)]
        default_value: Option<i32>,
        #[sea_orm(default = "String::from(\"DefaultExpr\")")]
        default_expr: Option<String>,
        #[sea_orm(ignore)]
        ignored: String,
        #[sea_orm(skip)]
        skipped: String,
    }

    assert_eq!(
        NewFruit {
            fruit_id: 7,
            fruit_name: Some("Apple".to_owned()),
            default_value: Some(10),
            default_expr: Some("Provided".to_owned()),
            ignored: "ignored".to_owned(),
            skipped: "skipped".to_owned(),
        }
        .into_active_model(),
        fruit::ActiveModel {
            id: Set(7),
            name: Set("Apple".to_owned()),
            cake_id: Set(Some(42)),
            default_value: Set(10),
            default_expr: Set("Provided".to_owned()),
        }
    );

    assert_eq!(
        NewFruit {
            fruit_id: 8,
            fruit_name: None,
            default_value: None,
            default_expr: None,
            ignored: "ignored".to_owned(),
            skipped: "skipped".to_owned(),
        }
        .into_active_model(),
        fruit::ActiveModel {
            id: Set(8),
            name: NotSet,
            cake_id: Set(Some(42)),
            default_value: Set(0),
            default_expr: Set("DefaultExpr".to_owned()),
        }
    );
}
