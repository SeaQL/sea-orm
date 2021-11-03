pub mod common;

pub use common::{features::*, setup::*, TestContext};

mod test_child_with_sd_via_sd_junction {
    use super::soft_delete_many_to_many::child_with_sd_via_sd_junction::*;
    use super::soft_delete_many_to_many::parent;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_related_eager() {
        let find_parent: Select<parent::Entity> = Entity::find_related();
        assert_eq!(
            find_parent
                .filter(Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `soft_delete_junction` ON `soft_delete_junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `soft_delete_child` ON `soft_delete_child`.`id` = `soft_delete_junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `soft_delete_junction`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_related_lazy() {
        let model = Model {
            id: 12,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_related(parent::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `soft_delete_junction` ON `soft_delete_junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `soft_delete_child` ON `soft_delete_child`.`id` = `soft_delete_junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `soft_delete_junction`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked() {
        assert_eq!(
            Entity::find()
                .find_also_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `soft_delete_child`.`id` AS `A_id`, `soft_delete_child`.`parent_id` AS `A_parent_id`, `soft_delete_child`.`name` AS `A_name`, `soft_delete_child`.`created_at` AS `A_created_at`, `soft_delete_child`.`updated_at` AS `A_updated_at`, `soft_delete_child`.`deleted_at` AS `A_deleted_at`,",
                "`r1`.`id` AS `B_id`, `r1`.`name` AS `B_name`, `r1`.`created_at` AS `B_created_at`, `r1`.`updated_at` AS `B_updated_at`, `r1`.`deleted_at` AS `B_deleted_at`",
                "FROM `soft_delete_child`",
                "LEFT JOIN `soft_delete_junction` AS `r0` ON `soft_delete_child`.`id` = `r0`.`child_id`",
                "LEFT JOIN `parent` AS `r1` ON `r0`.`parent_id` = `r1`.`id`",
                "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `r0`.`deleted_at` IS NULL",
                "AND `r1`.`deleted_at` IS NULL",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_linked() {
        let model = Model {
            id: 18,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `soft_delete_junction` AS `r0` ON `r0`.`parent_id` = `parent`.`id`",
                "INNER JOIN `soft_delete_child` AS `r1` ON `r1`.`id` = `r0`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `r0`.`deleted_at` IS NULL",
                "AND `r1`.`deleted_at` IS NULL",
                "AND `r1`.`id` = 18",
            ]
            .join(" ")
        );
    }
}

mod test_child_via_sd_junction {
    use super::soft_delete_many_to_many::child_via_sd_junction::*;
    use super::soft_delete_many_to_many::parent;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_related_eager() {
        let find_parent: Select<parent::Entity> = Entity::find_related();
        assert_eq!(
            find_parent
                .filter(Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `soft_delete_junction` ON `soft_delete_junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `child` ON `child`.`id` = `soft_delete_junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `soft_delete_junction`.`deleted_at` IS NULL",
                "AND `child`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_related_lazy() {
        let model = Model {
            id: 12,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_related(parent::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `soft_delete_junction` ON `soft_delete_junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `child` ON `child`.`id` = `soft_delete_junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `soft_delete_junction`.`deleted_at` IS NULL",
                "AND `child`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked() {
        assert_eq!(
            Entity::find()
                .find_also_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `child`.`id` AS `A_id`, `child`.`parent_id` AS `A_parent_id`, `child`.`name` AS `A_name`, `child`.`created_at` AS `A_created_at`, `child`.`updated_at` AS `A_updated_at`, `child`.`deleted_at` AS `A_deleted_at`,",
                "`r1`.`id` AS `B_id`, `r1`.`name` AS `B_name`, `r1`.`created_at` AS `B_created_at`, `r1`.`updated_at` AS `B_updated_at`, `r1`.`deleted_at` AS `B_deleted_at`",
                "FROM `child`",
                "LEFT JOIN `soft_delete_junction` AS `r0` ON `child`.`id` = `r0`.`child_id`",
                "LEFT JOIN `parent` AS `r1` ON `r0`.`parent_id` = `r1`.`id`",
                "WHERE `r1`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `r0`.`deleted_at` IS NULL",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_linked() {
        let model = Model {
            id: 18,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `soft_delete_junction` AS `r0` ON `r0`.`parent_id` = `parent`.`id`",
                "INNER JOIN `child` AS `r1` ON `r1`.`id` = `r0`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                // FIXME: No way to know if the junction have soft delete enabled with only RelationDef on hand
                // "AND `r0`.`deleted_at` IS NULL",
                "AND `r1`.`id` = 18",
            ]
            .join(" ")
        );
    }
}

mod test_child_with_sd_via_junction {
    use super::soft_delete_many_to_many::child_with_sd_via_junction::*;
    use super::soft_delete_many_to_many::parent;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_related_eager() {
        let find_parent: Select<parent::Entity> = Entity::find_related();
        assert_eq!(
            find_parent
                .filter(Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `junction` ON `junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `soft_delete_child` ON `soft_delete_child`.`id` = `junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_related_lazy() {
        let model = Model {
            id: 12,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_related(parent::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `junction` ON `junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `soft_delete_child` ON `soft_delete_child`.`id` = `junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`deleted_at` IS NULL",
                "AND `soft_delete_child`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked() {
        assert_eq!(
            Entity::find()
                .find_also_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `soft_delete_child`.`id` AS `A_id`, `soft_delete_child`.`parent_id` AS `A_parent_id`, `soft_delete_child`.`name` AS `A_name`, `soft_delete_child`.`created_at` AS `A_created_at`, `soft_delete_child`.`updated_at` AS `A_updated_at`, `soft_delete_child`.`deleted_at` AS `A_deleted_at`,",
                "`r1`.`id` AS `B_id`, `r1`.`name` AS `B_name`, `r1`.`created_at` AS `B_created_at`, `r1`.`updated_at` AS `B_updated_at`, `r1`.`deleted_at` AS `B_deleted_at`",
                "FROM `soft_delete_child`",
                "LEFT JOIN `junction` AS `r0` ON `soft_delete_child`.`id` = `r0`.`child_id`",
                "LEFT JOIN `parent` AS `r1` ON `r0`.`parent_id` = `r1`.`id`",
                "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                "AND `r1`.`deleted_at` IS NULL",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_linked() {
        let model = Model {
            id: 18,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `junction` AS `r0` ON `r0`.`parent_id` = `parent`.`id`",
                "INNER JOIN `soft_delete_child` AS `r1` ON `r1`.`id` = `r0`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `r1`.`deleted_at` IS NULL",
                "AND `r1`.`id` = 18",
            ]
            .join(" ")
        );
    }
}

mod test_child_via_junction {
    use super::soft_delete_many_to_many::child_via_junction::*;
    use super::soft_delete_many_to_many::parent;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_related_eager() {
        let find_parent: Select<parent::Entity> = Entity::find_related();
        assert_eq!(
            find_parent
                .filter(Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `junction` ON `junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `child` ON `child`.`id` = `junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `child`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_related_lazy() {
        let model = Model {
            id: 12,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_related(parent::Entity)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `junction` ON `junction`.`parent_id` = `parent`.`id`",
                "INNER JOIN `child` ON `child`.`id` = `junction`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `child`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked() {
        assert_eq!(
            Entity::find()
                .find_also_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `child`.`id` AS `A_id`, `child`.`parent_id` AS `A_parent_id`, `child`.`name` AS `A_name`, `child`.`created_at` AS `A_created_at`, `child`.`updated_at` AS `A_updated_at`, `child`.`deleted_at` AS `A_deleted_at`,",
                "`r1`.`id` AS `B_id`, `r1`.`name` AS `B_name`, `r1`.`created_at` AS `B_created_at`, `r1`.`updated_at` AS `B_updated_at`, `r1`.`deleted_at` AS `B_deleted_at`",
                "FROM `child`",
                "LEFT JOIN `junction` AS `r0` ON `child`.`id` = `r0`.`child_id`",
                "LEFT JOIN `parent` AS `r1` ON `r0`.`parent_id` = `r1`.`id`",
                "WHERE `r1`.`deleted_at` IS NULL",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_linked() {
        let model = Model {
            id: 18,
            parent_id: 1,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_linked(SoftDeleteChildToParent)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id`, `parent`.`name`, `parent`.`created_at`, `parent`.`updated_at`, `parent`.`deleted_at`",
                "FROM `parent`",
                "INNER JOIN `junction` AS `r0` ON `r0`.`parent_id` = `parent`.`id`",
                "INNER JOIN `child` AS `r1` ON `r1`.`id` = `r0`.`child_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `r1`.`id` = 18",
            ]
            .join(" ")
        );
    }
}
