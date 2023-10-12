pub mod json_string_vec {
    use sea_orm::entity::prelude::*;
    use sea_orm::FromJsonQueryResult;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "json_vec")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub str_vec: Option<StringVec>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
    pub struct StringVec(pub Vec<String>);
}

pub mod json_struct_vec {
    use sea_orm::entity::prelude::*;
    use sea_orm_macros::FromJsonQueryResult;
    use sea_query::with_array::NotU8;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
    pub struct JsonColumn {
        pub value: String,
    }

    impl NotU8 for JsonColumn {}

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "json_struct_vec")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_type = "JsonBinary")]
        pub struct_vec: Vec<JsonColumn>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
