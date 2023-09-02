pub mod string_vec {
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

pub mod struct_vec {
    use sea_orm::entity::prelude::*;
    use sea_orm_macros::FromJsonQueryResult;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
    pub struct JsonColumn {
        pub value: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "json_vec")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub struct_vec: Option<Vec<JsonColumn>>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
