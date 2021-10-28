pub mod parent {
    use super::child;
    use super::child_with_soft_delete;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "soft_delete")]
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

    impl ActiveModelBehavior for ActiveModel {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use pretty_assertions::assert_eq;
        use sea_orm::*;

        #[test]
        fn find_related_child_with_soft_delete_eager() {
            let find_child: Select<child_with_soft_delete::Entity> = Entity::find_related();
            assert_eq!(
                find_child
                    .filter(Column::Id.eq(11))
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `soft_delete_child`.`id`, `soft_delete_child`.`parent_id`, `soft_delete_child`.`name`, `soft_delete_child`.`created_at`, `soft_delete_child`.`updated_at`, `soft_delete_child`.`deleted_at`",
                    "FROM `soft_delete_child`",
                    "INNER JOIN `soft_delete` ON `soft_delete`.`id` = `soft_delete_child`.`parent_id`",
                    "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                    "AND `soft_delete`.`deleted_at` IS NULL",
                    "AND `soft_delete`.`id` = 11",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_related_child_with_soft_delete_lazy() {
            let model = Model {
                id: 12,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                model
                    .find_related(child_with_soft_delete::Entity)
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `soft_delete_child`.`id`, `soft_delete_child`.`parent_id`, `soft_delete_child`.`name`, `soft_delete_child`.`created_at`, `soft_delete_child`.`updated_at`, `soft_delete_child`.`deleted_at`",
                    "FROM `soft_delete_child`",
                    "INNER JOIN `soft_delete` ON `soft_delete`.`id` = `soft_delete_child`.`parent_id`",
                    "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                    "AND `soft_delete`.`deleted_at` IS NULL",
                    "AND `soft_delete`.`id` = 12",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_also_linked_child_with_soft_delete() {
            assert_eq!(
                Entity::find()
                    .find_also_linked(ParentToSoftDeleteChild)
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `soft_delete`.`id` AS `A_id`, `soft_delete`.`name` AS `A_name`, `soft_delete`.`created_at` AS `A_created_at`, `soft_delete`.`updated_at` AS `A_updated_at`, `soft_delete`.`deleted_at` AS `A_deleted_at`,",
                    "`r0`.`id` AS `B_id`, `r0`.`parent_id` AS `B_parent_id`, `r0`.`name` AS `B_name`, `r0`.`created_at` AS `B_created_at`, `r0`.`updated_at` AS `B_updated_at`, `r0`.`deleted_at` AS `B_deleted_at`",
                    "FROM `soft_delete`",
                    "LEFT JOIN `soft_delete_child` AS `r0` ON `soft_delete`.`id` = `r0`.`parent_id`",
                    "WHERE `soft_delete`.`deleted_at` IS NULL",
                    "AND `r0`.`deleted_at` IS NULL",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_linked_child_with_soft_delete() {
            let model = Model {
                id: 18,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                model
                    .find_linked(ParentToSoftDeleteChild)
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `soft_delete_child`.`id`, `soft_delete_child`.`parent_id`, `soft_delete_child`.`name`, `soft_delete_child`.`created_at`, `soft_delete_child`.`updated_at`, `soft_delete_child`.`deleted_at`",
                    "FROM `soft_delete_child`",
                    "INNER JOIN `soft_delete` AS `r0` ON `r0`.`id` = `soft_delete_child`.`parent_id`",
                    "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                    "AND `r0`.`deleted_at` IS NULL",
                    "AND `r0`.`id` = 18",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_related_child_eager() {
            let find_child: Select<child::Entity> = Entity::find_related();
            assert_eq!(
                find_child
                    .filter(Column::Id.eq(11))
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `child`.`id`, `child`.`parent_id`, `child`.`name`, `child`.`created_at`, `child`.`updated_at`, `child`.`deleted_at`",
                    "FROM `child`",
                    "INNER JOIN `soft_delete` ON `soft_delete`.`id` = `child`.`parent_id`",
                    "WHERE `soft_delete`.`deleted_at` IS NULL",
                    "AND `soft_delete`.`id` = 11",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_related_child_lazy() {
            let model = Model {
                id: 12,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                model
                    .find_related(child::Entity)
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `child`.`id`, `child`.`parent_id`, `child`.`name`, `child`.`created_at`, `child`.`updated_at`, `child`.`deleted_at`",
                    "FROM `child`",
                    "INNER JOIN `soft_delete` ON `soft_delete`.`id` = `child`.`parent_id`",
                    "WHERE `soft_delete`.`deleted_at` IS NULL",
                    "AND `soft_delete`.`id` = 12",
                ]
                .join(" ")
            );
        }
    }
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
