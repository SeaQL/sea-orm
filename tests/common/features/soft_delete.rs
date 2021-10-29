pub mod model_with_soft_delete {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "soft_delete_model")]
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::offset::Local;
        use pretty_assertions::assert_eq;
        use sea_orm::*;

        #[test]
        fn find() {
            assert_eq!(
                Entity::find()
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `soft_delete_model`.`id`, `soft_delete_model`.`name`, `soft_delete_model`.`created_at`, `soft_delete_model`.`updated_at`, `soft_delete_model`.`deleted_at`",
                    "FROM `soft_delete_model`",
                    "WHERE `soft_delete_model`.`deleted_at` IS NULL",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_with_deleted() {
            assert_eq!(
                Entity::find_with_deleted()
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `soft_delete_model`.`id`, `soft_delete_model`.`name`, `soft_delete_model`.`created_at`, `soft_delete_model`.`updated_at`, `soft_delete_model`.`deleted_at`",
                    "FROM `soft_delete_model`",
                ]
                .join(" ")
            );
        }

        #[test]
        fn delete_one() {
            let model = Model {
                id: 12,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                Entity::delete(model.into_active_model())
                    .build(DbBackend::MySql)
                    .to_string(),
                format!(
                    "UPDATE `soft_delete_model` SET `deleted_at` = '{}' WHERE `soft_delete_model`.`id` = 12",
                    Local::now().naive_local().format("%Y-%m-%d %H:%M:%S")
                )
            );
        }

        #[test]
        fn delete_many() {
            assert_eq!(
                Entity::delete_many()
                    .filter(Column::Id.eq(12))
                    .build(DbBackend::MySql)
                    .to_string(),
                format!(
                    "UPDATE `soft_delete_model` SET `deleted_at` = '{}' WHERE `soft_delete_model`.`id` = 12",
                    Local::now().naive_local().format("%Y-%m-%d %H:%M:%S")
                )
            );
        }

        #[test]
        fn delete_one_forcefully() {
            let model = Model {
                id: 12,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                Entity::delete_forcefully(model.into_active_model())
                    .build(DbBackend::MySql)
                    .to_string()
                    .as_str(),
                "DELETE FROM `soft_delete_model` WHERE `soft_delete_model`.`id` = 12",
            );
        }

        #[test]
        fn delete_many_forcefully() {
            assert_eq!(
                Entity::delete_many_forcefully()
                    .filter(Column::Id.eq(12))
                    .build(DbBackend::MySql)
                    .to_string()
                    .as_str(),
                "DELETE FROM `soft_delete_model` WHERE `soft_delete_model`.`id` = 12",
            );
        }
    }
}
pub mod model {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "model")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub created_at: Option<DateTime>,
        pub updated_at: Option<DateTime>,
        pub deleted_at: Option<DateTime>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    #[cfg(test)]
    mod tests {
        use super::*;
        use pretty_assertions::assert_eq;
        use sea_orm::*;

        #[test]
        fn find() {
            assert_eq!(
                Entity::find()
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `model`.`id`, `model`.`name`, `model`.`created_at`, `model`.`updated_at`, `model`.`deleted_at`",
                    "FROM `model`",
                ]
                .join(" ")
            );
        }

        #[test]
        fn find_with_deleted() {
            assert_eq!(
                Entity::find_with_deleted()
                    .build(DbBackend::MySql)
                    .to_string(),
                [
                    "SELECT `model`.`id`, `model`.`name`, `model`.`created_at`, `model`.`updated_at`, `model`.`deleted_at`",
                    "FROM `model`",
                ]
                .join(" ")
            );
        }

        #[test]
        fn delete_one() {
            let model = Model {
                id: 12,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                Entity::delete(model.into_active_model())
                    .build(DbBackend::MySql)
                    .to_string()
                    .as_str(),
                "DELETE FROM `model` WHERE `model`.`id` = 12",
            );
        }

        #[test]
        fn delete_many() {
            assert_eq!(
                Entity::delete_many()
                    .filter(Column::Id.eq(12))
                    .build(DbBackend::MySql)
                    .to_string()
                    .as_str(),
                "DELETE FROM `model` WHERE `model`.`id` = 12",
            );
        }

        #[test]
        fn delete_one_forcefully() {
            let model = Model {
                id: 12,
                name: "".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            };

            assert_eq!(
                Entity::delete_forcefully(model.into_active_model())
                    .build(DbBackend::MySql)
                    .to_string()
                    .as_str(),
                "DELETE FROM `model` WHERE `model`.`id` = 12",
            );
        }

        #[test]
        fn delete_many_forcefully() {
            assert_eq!(
                Entity::delete_many_forcefully()
                    .filter(Column::Id.eq(12))
                    .build(DbBackend::MySql)
                    .to_string()
                    .as_str(),
                "DELETE FROM `model` WHERE `model`.`id` = 12",
            );
        }
    }
}
