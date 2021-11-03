pub mod parent {
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
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod junction_with_soft_delete {
    use super::child_via_sd_junction;
    use super::child_with_sd_via_sd_junction;
    use super::parent;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "soft_delete_junction")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub parent_id: i32,
        pub child_id: i32,
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
        #[sea_orm(
            belongs_to = "child_via_sd_junction::Entity",
            from = "Column::ChildId",
            to = "child_via_sd_junction::Column::Id"
        )]
        ChildViaSoftDeleteJunction,
        #[sea_orm(
            belongs_to = "child_with_sd_via_sd_junction::Entity",
            from = "Column::ChildId",
            to = "child_with_sd_via_sd_junction::Column::Id"
        )]
        SoftDeleteChildViaSoftDeleteJunction,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod junction {
    use super::child_via_junction;
    use super::child_with_sd_via_junction;
    use super::parent;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "junction")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub parent_id: i32,
        pub child_id: i32,
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
        #[sea_orm(
            belongs_to = "child_via_junction::Entity",
            from = "Column::ChildId",
            to = "child_via_junction::Column::Id"
        )]
        ChildViaJunction,
        #[sea_orm(
            belongs_to = "child_with_sd_via_junction::Entity",
            from = "Column::ChildId",
            to = "child_with_sd_via_junction::Column::Id"
        )]
        SoftDeleteChildViaJunction,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod child_with_sd_via_sd_junction {
    use super::junction_with_soft_delete;
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
    pub enum Relation {}

    impl Related<parent::Entity> for Entity {
        fn to() -> RelationDef {
            junction_with_soft_delete::Relation::Parent.def()
        }

        fn via() -> Option<RelationDef> {
            Some(
                junction_with_soft_delete::Relation::SoftDeleteChildViaSoftDeleteJunction
                    .def()
                    .rev(),
            )
        }
    }

    pub struct SoftDeleteChildToParent;

    impl Linked for SoftDeleteChildToParent {
        type FromEntity = Entity;

        type ToEntity = parent::Entity;

        fn link(&self) -> Vec<RelationDef> {
            vec![
                junction_with_soft_delete::Relation::SoftDeleteChildViaSoftDeleteJunction
                    .def()
                    .rev(),
                junction_with_soft_delete::Relation::Parent.def(),
            ]
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod child_via_sd_junction {
    use super::junction_with_soft_delete;
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
    pub enum Relation {}

    impl Related<parent::Entity> for Entity {
        fn to() -> RelationDef {
            junction_with_soft_delete::Relation::Parent.def()
        }

        fn via() -> Option<RelationDef> {
            Some(
                junction_with_soft_delete::Relation::ChildViaSoftDeleteJunction
                    .def()
                    .rev(),
            )
        }
    }

    pub struct SoftDeleteChildToParent;

    impl Linked for SoftDeleteChildToParent {
        type FromEntity = Entity;

        type ToEntity = parent::Entity;

        fn link(&self) -> Vec<RelationDef> {
            vec![
                junction_with_soft_delete::Relation::ChildViaSoftDeleteJunction
                    .def()
                    .rev(),
                junction_with_soft_delete::Relation::Parent.def(),
            ]
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod child_with_sd_via_junction {
    use super::junction;
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
    pub enum Relation {}

    impl Related<parent::Entity> for Entity {
        fn to() -> RelationDef {
            junction::Relation::Parent.def()
        }

        fn via() -> Option<RelationDef> {
            Some(junction::Relation::SoftDeleteChildViaJunction.def().rev())
        }
    }

    pub struct SoftDeleteChildToParent;

    impl Linked for SoftDeleteChildToParent {
        type FromEntity = Entity;

        type ToEntity = parent::Entity;

        fn link(&self) -> Vec<RelationDef> {
            vec![
                junction::Relation::SoftDeleteChildViaJunction.def().rev(),
                junction::Relation::Parent.def(),
            ]
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod child_via_junction {
    use super::junction;
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
    pub enum Relation {}

    impl Related<parent::Entity> for Entity {
        fn to() -> RelationDef {
            junction::Relation::Parent.def()
        }

        fn via() -> Option<RelationDef> {
            Some(junction::Relation::ChildViaJunction.def().rev())
        }
    }

    pub struct SoftDeleteChildToParent;

    impl Linked for SoftDeleteChildToParent {
        type FromEntity = Entity;

        type ToEntity = parent::Entity;

        fn link(&self) -> Vec<RelationDef> {
            vec![
                junction::Relation::ChildViaJunction.def().rev(),
                junction::Relation::Parent.def(),
            ]
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
