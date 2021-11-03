pub mod common;

use chrono::offset::Local;
pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, *};
use sea_query::{ColumnDef, ForeignKeyCreateStatement, Table};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    test_soft_deletes().await?;

    Ok(())
}

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    create_parent_table(db).await?;
    create_child_with_soft_delete_table(db).await?;
    create_child_table(db).await?;

    Ok(())
}

pub async fn create_parent_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use soft_delete_one_to_many::parent::*;

    let stmt = Table::create()
        .table(Entity)
        .col(
            ColumnDef::new(Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Column::Name).string().not_null())
        .col(ColumnDef::new(Column::CreatedAt).date_time())
        .col(ColumnDef::new(Column::UpdatedAt).date_time())
        .col(ColumnDef::new(Column::DeletedAt).date_time())
        .to_owned();

    create_table(db, &stmt, Entity).await
}

pub async fn create_child_with_soft_delete_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use soft_delete_one_to_many::child_with_soft_delete::*;
    use soft_delete_one_to_many::parent;

    let stmt = Table::create()
        .table(Entity)
        .col(
            ColumnDef::new(Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Column::ParentId).integer().not_null())
        .col(ColumnDef::new(Column::Name).string().not_null())
        .col(ColumnDef::new(Column::CreatedAt).date_time())
        .col(ColumnDef::new(Column::UpdatedAt).date_time())
        .col(ColumnDef::new(Column::DeletedAt).date_time())
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-soft_delete_child-parent")
                .from_tbl(Entity)
                .from_col(Column::ParentId)
                .to_tbl(parent::Entity)
                .to_col(parent::Column::Id),
        )
        .to_owned();

    create_table(db, &stmt, Entity).await
}

pub async fn create_child_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    use soft_delete_one_to_many::child::*;
    use soft_delete_one_to_many::parent;

    let stmt = Table::create()
        .table(Entity)
        .col(
            ColumnDef::new(Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Column::ParentId).integer().not_null())
        .col(ColumnDef::new(Column::Name).string().not_null())
        .col(ColumnDef::new(Column::CreatedAt).date_time())
        .col(ColumnDef::new(Column::UpdatedAt).date_time())
        .col(ColumnDef::new(Column::DeletedAt).date_time())
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-child-parent")
                .from_tbl(Entity)
                .from_col(Column::ParentId)
                .to_tbl(parent::Entity)
                .to_col(parent::Column::Id),
        )
        .to_owned();

    create_table(db, &stmt, Entity).await
}

pub async fn test_soft_deletes() -> Result<(), DbErr> {
    let ctx = TestContext::new("soft_delete_one_to_many_tests_1").await;
    create_tables(&ctx.db).await?;
    create_and_delete_parent_child_with_soft_delete(&ctx.db).await?;
    ctx.delete().await;

    let ctx = TestContext::new("soft_delete_one_to_many_tests_2").await;
    create_tables(&ctx.db).await?;
    create_and_delete_parent_child(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

macro_rules! count_any {
    ( $mod: ident, $fn: ident, $db: ident ) => {
        $mod::Entity::$fn().count($db).await?
    };
}

macro_rules! count {
    ( $mod: ident, $db: ident ) => {
        count_any!($mod, find, $db)
    };
}

macro_rules! count_with_deleted {
    ( $mod: ident, $db: ident ) => {
        count_any!($mod, find_with_deleted, $db)
    };
}

// Testing parent-child model with soft delete enabled
pub async fn create_and_delete_parent_child_with_soft_delete(
    db: &DatabaseConnection,
) -> Result<(), DbErr> {
    use soft_delete_one_to_many::child_with_soft_delete as child;
    use soft_delete_one_to_many::parent;

    // Insert 10 parent models each with i-th number of children, and count the number of rows
    let num_parent = 10;
    let num_child = num_parent * (num_parent + 1) / 2;
    for i in 1..=num_parent {
        let parent = parent::ActiveModel {
            name: Set(format!("Parent Model {}", i)),
            ..Default::default()
        }
        .save(db)
        .await?;

        for j in 1..=i {
            child::ActiveModel {
                parent_id: Set(parent.id.clone().unwrap()),
                name: Set(format!("Child Model {}.{}", i, j)),
                ..Default::default()
            }
            .save(db)
            .await?;
        }

        assert_eq!(count!(parent, db), i);
        assert_eq!(count!(child, db), i * (i + 1) / 2);
    }
    assert_eq!(count!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child);

    // Retrieve the first parent model
    let parent = parent::Entity::find()
        .order_by_asc(parent::Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        parent,
        parent::Model {
            id: 1,
            name: "Parent Model 1".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Soft delete the retrieved parent model, and make sure it's soft deleted
    parent.into_active_model().delete(db).await?;
    assert_eq!(count!(parent, db), num_parent - 1);
    assert_eq!(count_with_deleted!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child);
    assert_eq!(count_with_deleted!(child, db), num_child);

    // Retrieve the soft deleted parent model
    let soft_deleted_parent = parent::Entity::find_with_deleted()
        .order_by_asc(parent::Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert!(soft_deleted_parent.deleted_at.is_some());
    // Find all of the children
    assert_eq!(
        soft_deleted_parent
            .find_related(child::Entity)
            .all(db)
            .await?,
        vec![]
    );

    // Retrieve first child of the soft deleted parent model
    let child = child::Entity::find()
        .order_by_asc(child::Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        child,
        child::Model {
            id: 1,
            parent_id: 1,
            name: "Child Model 1.1".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );
    // Find its parent
    assert_eq!(child.find_related(parent::Entity).all(db).await?, vec![]);

    // Retrieve the second parent model and its children via `find_also_related`
    let parent = parent::Model {
        id: 2,
        name: "Parent Model 2".to_owned(),
        created_at: None,
        updated_at: None,
        deleted_at: None,
    };
    let find_also_related: Vec<(parent::Model, Option<child::Model>)> = parent::Entity::find()
        .find_also_related(child::Entity)
        .filter(parent::Column::Id.eq(2))
        .all(db)
        .await?;
    assert_eq!(
        find_also_related,
        vec![
            (
                parent.clone(),
                Some(child::Model {
                    id: 2,
                    parent_id: 2,
                    name: "Child Model 2.1".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                })
            ),
            (
                parent.clone(),
                Some(child::Model {
                    id: 3,
                    parent_id: 2,
                    name: "Child Model 2.2".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                })
            ),
        ]
    );

    // Retrieve the second parent model and its children via `find_with_related`
    assert_eq!(
        parent::Entity::find()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(
            parent.clone(),
            vec![
                child::Model {
                    id: 2,
                    parent_id: 2,
                    name: "Child Model 2.1".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                },
                child::Model {
                    id: 3,
                    parent_id: 2,
                    name: "Child Model 2.2".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                },
            ]
        )]
    );

    // Soft delete the first child
    let first_child = find_also_related[0].1.clone().unwrap();
    first_child.into_active_model().delete(db).await?;
    assert_eq!(count!(parent, db), num_parent - 1);
    assert_eq!(count_with_deleted!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child - 1);
    assert_eq!(count_with_deleted!(child, db), num_child);

    // Retrieve the second parent model and its children to double check we have soft deleted first child
    assert_eq!(
        parent::Entity::find()
            .find_also_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(
            parent.clone(),
            Some(child::Model {
                id: 3,
                parent_id: 2,
                name: "Child Model 2.2".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            })
        )]
    );
    assert_eq!(
        parent::Entity::find()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(
            parent.clone(),
            vec![child::Model {
                id: 3,
                parent_id: 2,
                name: "Child Model 2.2".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            }]
        )]
    );

    // Force delete the second child
    let second_child = find_also_related[1].1.clone().unwrap();
    second_child
        .into_active_model()
        .delete_forcefully(db)
        .await?;
    assert_eq!(count!(parent, db), num_parent - 1);
    assert_eq!(count_with_deleted!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child - 2);
    assert_eq!(count_with_deleted!(child, db), num_child - 1);

    // Retrieve the second parent model and its children to double check
    let soft_deleted_child = child::Entity::find_with_deleted()
        .filter(child::Column::Id.eq(2))
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        parent::Entity::find()
            .find_also_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![]
    );
    assert_eq!(
        parent::Entity::find_with_deleted()
            .find_also_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(parent.clone(), Some(soft_deleted_child.clone()))]
    );
    assert_eq!(
        parent::Entity::find()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![]
    );
    assert_eq!(
        parent::Entity::find_with_deleted()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(parent.clone(), vec![soft_deleted_child.clone()])]
    );

    Ok(())
}

// Testing parent-child model with soft delete disabled
pub async fn create_and_delete_parent_child(db: &DatabaseConnection) -> Result<(), DbErr> {
    #[allow(unused_imports)]
    use soft_delete_one_to_many::child;
    use soft_delete_one_to_many::parent;

    // Insert 10 parent models each with i-th number of children, and count the number of rows
    let num_parent = 10;
    let num_child = num_parent * (num_parent + 1) / 2;
    for i in 1..=num_parent {
        let parent = parent::ActiveModel {
            name: Set(format!("Parent Model {}", i)),
            ..Default::default()
        }
        .save(db)
        .await?;

        for j in 1..=i {
            child::ActiveModel {
                parent_id: Set(parent.id.clone().unwrap()),
                name: Set(format!("Child Model {}.{}", i, j)),
                ..Default::default()
            }
            .save(db)
            .await?;
        }

        assert_eq!(count!(parent, db), i);
        assert_eq!(count!(child, db), i * (i + 1) / 2);
    }
    assert_eq!(count!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child);

    // Retrieve the first parent model
    let parent = parent::Entity::find()
        .order_by_asc(parent::Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        parent,
        parent::Model {
            id: 1,
            name: "Parent Model 1".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );

    // Delete the retrieved parent model, and make sure it's soft deleted
    parent.into_active_model().delete(db).await?;
    assert_eq!(count!(parent, db), num_parent - 1);
    assert_eq!(count_with_deleted!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child);
    assert_eq!(count_with_deleted!(child, db), num_child);

    // Retrieve the soft deleted parent model
    let soft_deleted_parent = parent::Entity::find_with_deleted()
        .order_by_asc(parent::Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert!(soft_deleted_parent.deleted_at.is_some());
    // Find all of the children
    assert_eq!(
        soft_deleted_parent
            .find_related(child::Entity)
            .all(db)
            .await?,
        vec![]
    );

    // Retrieve first child of the soft deleted parent model
    let child = child::Entity::find()
        .order_by_asc(child::Column::Id)
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        child,
        child::Model {
            id: 1,
            parent_id: 1,
            name: "Child Model 1.1".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        }
    );
    // Find its parent
    assert_eq!(child.find_related(parent::Entity).all(db).await?, vec![]);

    // Retrieve the second parent model and its children via `find_also_related`
    let parent = parent::Model {
        id: 2,
        name: "Parent Model 2".to_owned(),
        created_at: None,
        updated_at: None,
        deleted_at: None,
    };
    let find_also_related: Vec<(parent::Model, Option<child::Model>)> = parent::Entity::find()
        .find_also_related(child::Entity)
        .filter(parent::Column::Id.eq(2))
        .all(db)
        .await?;
    assert_eq!(
        find_also_related,
        vec![
            (
                parent.clone(),
                Some(child::Model {
                    id: 2,
                    parent_id: 2,
                    name: "Child Model 2.1".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                })
            ),
            (
                parent.clone(),
                Some(child::Model {
                    id: 3,
                    parent_id: 2,
                    name: "Child Model 2.2".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                })
            ),
        ]
    );

    // Retrieve the second parent model and its children via `find_with_related`
    assert_eq!(
        parent::Entity::find()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(
            parent.clone(),
            vec![
                child::Model {
                    id: 2,
                    parent_id: 2,
                    name: "Child Model 2.1".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                },
                child::Model {
                    id: 3,
                    parent_id: 2,
                    name: "Child Model 2.2".to_owned(),
                    created_at: None,
                    updated_at: None,
                    deleted_at: None,
                },
            ]
        )]
    );

    // Delete the first child
    let first_child = find_also_related[0].1.clone().unwrap();
    first_child.into_active_model().delete(db).await?;
    assert_eq!(count!(parent, db), num_parent - 1);
    assert_eq!(count_with_deleted!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child - 1);
    assert_eq!(count_with_deleted!(child, db), num_child - 1);

    // Retrieve the second parent model and its children to double check we have deleted first child
    assert_eq!(
        parent::Entity::find()
            .find_also_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(
            parent.clone(),
            Some(child::Model {
                id: 3,
                parent_id: 2,
                name: "Child Model 2.2".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            })
        )]
    );
    assert_eq!(
        parent::Entity::find()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(
            parent.clone(),
            vec![child::Model {
                id: 3,
                parent_id: 2,
                name: "Child Model 2.2".to_owned(),
                created_at: None,
                updated_at: None,
                deleted_at: None,
            }]
        )]
    );

    // Try soft delete the second child
    let second_child = find_also_related[1].1.clone().unwrap();
    child::ActiveModel {
        deleted_at: Set(Some(Local::now().naive_local())),
        ..second_child.into_active_model()
    }
    .save(db)
    .await?;
    assert_eq!(count!(parent, db), num_parent - 1);
    assert_eq!(count_with_deleted!(parent, db), num_parent);
    assert_eq!(count!(child, db), num_child - 1);
    assert_eq!(count_with_deleted!(child, db), num_child - 1);

    // Retrieve the second parent model and its children to double check
    let soft_deleted_child = child::Entity::find_with_deleted()
        .filter(child::Column::Id.eq(3))
        .one(db)
        .await?
        .unwrap();
    assert_eq!(
        parent::Entity::find()
            .find_also_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(parent.clone(), Some(soft_deleted_child.clone()))]
    );
    assert_eq!(
        parent::Entity::find_with_deleted()
            .find_also_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(parent.clone(), Some(soft_deleted_child.clone()))]
    );
    assert_eq!(
        parent::Entity::find()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(parent.clone(), vec![soft_deleted_child.clone()])]
    );
    assert_eq!(
        parent::Entity::find_with_deleted()
            .find_with_related(child::Entity)
            .filter(parent::Column::Id.eq(2))
            .all(db)
            .await?,
        vec![(parent.clone(), vec![soft_deleted_child.clone()])]
    );

    Ok(())
}

mod test_parent_to_child_with_soft_delete {
    use super::soft_delete_one_to_many::child_with_soft_delete as child;
    use super::soft_delete_one_to_many::parent::*;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_related_eager() {
        let find_child: Select<child::Entity> = Entity::find_related();
        assert_eq!(
            find_child
                .filter(Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `soft_delete_child`.`id`, `soft_delete_child`.`parent_id`, `soft_delete_child`.`name`, `soft_delete_child`.`created_at`, `soft_delete_child`.`updated_at`, `soft_delete_child`.`deleted_at`",
                "FROM `soft_delete_child`",
                "INNER JOIN `parent` ON `parent`.`id` = `soft_delete_child`.`parent_id`",
                "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                "AND `parent`.`deleted_at` IS NULL",
                "AND `parent`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_related_lazy() {
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
                "SELECT `soft_delete_child`.`id`, `soft_delete_child`.`parent_id`, `soft_delete_child`.`name`, `soft_delete_child`.`created_at`, `soft_delete_child`.`updated_at`, `soft_delete_child`.`deleted_at`",
                "FROM `soft_delete_child`",
                "INNER JOIN `parent` ON `parent`.`id` = `soft_delete_child`.`parent_id`",
                "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                "AND `parent`.`deleted_at` IS NULL",
                "AND `parent`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked() {
        assert_eq!(
            Entity::find()
                .find_also_linked(ParentToSoftDeleteChild)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id` AS `A_id`, `parent`.`name` AS `A_name`, `parent`.`created_at` AS `A_created_at`, `parent`.`updated_at` AS `A_updated_at`, `parent`.`deleted_at` AS `A_deleted_at`,",
                "`r0`.`id` AS `B_id`, `r0`.`parent_id` AS `B_parent_id`, `r0`.`name` AS `B_name`, `r0`.`created_at` AS `B_created_at`, `r0`.`updated_at` AS `B_updated_at`, `r0`.`deleted_at` AS `B_deleted_at`",
                "FROM `parent`",
                "LEFT JOIN `soft_delete_child` AS `r0` ON `parent`.`id` = `r0`.`parent_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `r0`.`deleted_at` IS NULL",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_linked() {
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
                "INNER JOIN `parent` AS `r0` ON `r0`.`id` = `soft_delete_child`.`parent_id`",
                "WHERE `soft_delete_child`.`deleted_at` IS NULL",
                "AND `r0`.`deleted_at` IS NULL",
                "AND `r0`.`id` = 18",
            ]
            .join(" ")
        );
    }
}

mod tests_parent_to_child {
    use super::soft_delete_one_to_many::child;
    use super::soft_delete_one_to_many::parent::*;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_related_eager() {
        let find_child: Select<child::Entity> = Entity::find_related();
        assert_eq!(
            find_child
                .filter(Column::Id.eq(11))
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `child`.`id`, `child`.`parent_id`, `child`.`name`, `child`.`created_at`, `child`.`updated_at`, `child`.`deleted_at`",
                "FROM `child`",
                "INNER JOIN `parent` ON `parent`.`id` = `child`.`parent_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `parent`.`id` = 11",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_related_lazy() {
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
                "INNER JOIN `parent` ON `parent`.`id` = `child`.`parent_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
                "AND `parent`.`id` = 12",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked() {
        assert_eq!(
            Entity::find()
                .find_also_linked(ParentToChild)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `parent`.`id` AS `A_id`, `parent`.`name` AS `A_name`, `parent`.`created_at` AS `A_created_at`, `parent`.`updated_at` AS `A_updated_at`, `parent`.`deleted_at` AS `A_deleted_at`,",
                "`r0`.`id` AS `B_id`, `r0`.`parent_id` AS `B_parent_id`, `r0`.`name` AS `B_name`, `r0`.`created_at` AS `B_created_at`, `r0`.`updated_at` AS `B_updated_at`, `r0`.`deleted_at` AS `B_deleted_at`",
                "FROM `parent`",
                "LEFT JOIN `child` AS `r0` ON `parent`.`id` = `r0`.`parent_id`",
                "WHERE `parent`.`deleted_at` IS NULL",
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_linked() {
        let model = Model {
            id: 18,
            name: "".to_owned(),
            created_at: None,
            updated_at: None,
            deleted_at: None,
        };

        assert_eq!(
            model
                .find_linked(ParentToChild)
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `child`.`id`, `child`.`parent_id`, `child`.`name`, `child`.`created_at`, `child`.`updated_at`, `child`.`deleted_at`",
                "FROM `child`",
                "INNER JOIN `parent` AS `r0` ON `r0`.`id` = `child`.`parent_id`",
                "WHERE `r0`.`deleted_at` IS NULL",
                "AND `r0`.`id` = 18",
            ]
            .join(" ")
        );
    }
}
