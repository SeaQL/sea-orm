pub mod parent {
    use super::child;
    use super::child_with_soft_delete;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "parent")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub created_at: Option<DateTime>,
        pub updated_at: Option<DateTime>,
        #[sea_orm(soft_delete_column)]
        pub deleted_at: Option<DateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "child_with_soft_delete::Entity")]
        SoftDeleteChild,
        #[sea_orm(has_many = "child::Entity")]
        Child,
    }

    impl Related<child_with_soft_delete::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::SoftDeleteChild.def()
        }
    }

    impl Related<child::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Child.def()
        }
    }

    pub struct ParentToSoftDeleteChild;

    impl Linked for ParentToSoftDeleteChild {
        type FromEntity = Entity;

        type ToEntity = child_with_soft_delete::Entity;

        fn link(&self) -> Vec<RelationDef> {
            vec![Relation::SoftDeleteChild.def()]
        }
    }

    pub struct ParentToChild;

    impl Linked for ParentToChild {
        type FromEntity = Entity;

        type ToEntity = child::Entity;

        fn link(&self) -> Vec<RelationDef> {
            vec![Relation::Child.def()]
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod child_with_soft_delete {
    use super::parent;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "soft_delete_child")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub parent_id: i32,
        pub name: String,
        pub created_at: Option<DateTime>,
        pub updated_at: Option<DateTime>,
        #[sea_orm(soft_delete_column)]
        pub deleted_at: Option<DateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "parent::Entity",
            from = "Column::ParentId",
            to = "parent::Column::Id"
        )]
        Parent,
    }

    impl Related<parent::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Parent.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod child {
    use super::parent;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "child")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub parent_id: i32,
        pub name: String,
        pub created_at: Option<DateTime>,
        pub updated_at: Option<DateTime>,
        pub deleted_at: Option<DateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "parent::Entity",
            from = "Column::ParentId",
            to = "parent::Column::Id"
        )]
        Parent,
    }

    impl Related<parent::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Parent.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
