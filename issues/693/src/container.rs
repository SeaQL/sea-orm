pub mod prelude {
    pub use super::model::{
        ActiveModel as ContainerActiveModel, Column as ContainerColumn, Entity as Container,
        Model as ContainerModel, PrimaryKey as ContainerPrimaryKey, Relation as ContainerRelation,
    };
}

pub mod model {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "container")]
    pub struct Model {
        #[sea_orm(primary_key, column_name = "db_id")]
        pub rust_id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "crate::Content")]
        Content, // 1(Container) ⇆ n(Content)
    }

    impl Related<crate::Content> for Entity {
        fn to() -> RelationDef {
            Relation::Content.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
