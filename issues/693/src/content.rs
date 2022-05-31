pub mod prelude {
    pub use super::model::{
        ActiveModel as ContentActiveModel, Column as ContentColumn, Entity as Content,
        Model as ContentModel, PrimaryKey as ContentPrimaryKey, Relation as ContentRelation,
    };
}

pub mod model {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "content")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub container_id: i32,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "crate::Container",
            from = "crate::ContentColumn::ContainerId",
            to = "crate::ContainerColumn::RustId"
        )]
        Container, // 1(Container) ⇆ n(Content)
    }

    impl Related<crate::Container> for Entity {
        fn to() -> RelationDef {
            Relation::Container.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
