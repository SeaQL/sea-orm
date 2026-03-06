use sea_orm::prelude::{HasMany, HasOne};

mod cake {
    use sea_orm::prelude::*;
    use serde::Serialize;

    #[sea_orm::model]
    #[derive(DeriveEntityModel, Debug, Clone, Serialize)]
    #[sea_orm(table_name = "cake")]
    #[sea_orm(model_attrs(serde(rename_all = "UPPERCASE")))]
    #[sea_orm(model_ex_attrs(serde(rename_all = "PascalCase")))]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(has_many)]
        pub fruits: HasMany<super::fruit::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod fruit {
    use sea_orm::prelude::*;
    use serde::Serialize;

    #[sea_orm::model]
    #[derive(DeriveEntityModel, Debug, Clone)]
    #[sea_orm(
        table_name = "fruit",
        model_attrs(derive(Serialize), serde(rename_all = "UPPERCASE")),
        model_ex_attrs(derive(Serialize), serde(rename_all = "PascalCase"))
    )]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub cake_id: Option<i32>,
        #[sea_orm(belongs_to, from = "cake_id", to = "id")]
        pub cake: HasOne<super::cake::Entity>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn main() -> Result<(), serde_json::Error> {
    use sea_orm::EntityName;
    assert_eq!(cake::Entity.table_name(), "cake");
    assert_eq!(fruit::Entity.table_name(), "fruit");

    assert_eq!(serde_json::to_string(&cake::Model { id: 1 })?, "{\"ID\":1}");
    assert_eq!(
        serde_json::to_string(&cake::ModelEx {
            id: 1,
            fruits: HasMany::Loaded(Vec::new()),
        })?,
        "{\"Id\":1,\"Fruits\":[]}"
    );

    assert_eq!(
        serde_json::to_string(&fruit::Model {
            id: 2,
            cake_id: Some(1)
        })?,
        "{\"ID\":2,\"CAKE_ID\":1}"
    );
    assert_eq!(
        serde_json::to_string(&fruit::ModelEx {
            id: 2,
            cake_id: Some(1),
            cake: HasOne::Unloaded,
        })?,
        "{\"Id\":2,\"CakeId\":1,\"Cake\":null}"
    );

    Ok(())
}
