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
                "SELECT `soft_delete`.`id`, `soft_delete`.`name`, `soft_delete`.`created_at`, `soft_delete`.`updated_at`, `soft_delete`.`deleted_at`",
                "FROM `soft_delete`",
                "WHERE `soft_delete`.`deleted_at` IS NULL",
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
                "SELECT `soft_delete`.`id`, `soft_delete`.`name`, `soft_delete`.`created_at`, `soft_delete`.`updated_at`, `soft_delete`.`deleted_at`",
                "FROM `soft_delete`",
            ]
            .join(" ")
        );
    }
}
