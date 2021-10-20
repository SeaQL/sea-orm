use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "self_join")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub uuid_ref: Option<Uuid>,
    pub time: Option<Time>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity", from = "Column::UuidRef", to = "Column::Uuid")]
    SelfReferencing,
}

pub struct SelfReferencingLink;

impl Linked for SelfReferencingLink {
    type FromEntity = Entity;

    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::SelfReferencing.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use sea_orm::*;

    #[test]
    fn find_linked_001() {
        let self_join_model = Model {
            uuid: Uuid::default(),
            uuid_ref: None,
            time: None,
        };

        assert_eq!(
            self_join_model
                .find_linked(SelfReferencingLink)
                .build(DbBackend::MySql)
                .to_string(),
            [
                r#"SELECT `self_join`.`uuid`, `self_join`.`uuid_ref`, `self_join`.`time`"#,
                r#"FROM `self_join`"#,
                r#"INNER JOIN `self_join` AS `r0` ON `r0`.`uuid_ref` = `self_join`.`uuid`"#,
                r#"WHERE `r0`.`uuid` = '00000000-0000-0000-0000-000000000000'"#,
            ]
            .join(" ")
        );
    }

    #[test]
    fn find_also_linked_001() {
        assert_eq!(
            Entity::find()
                .find_also_linked(SelfReferencingLink)
                .build(DbBackend::MySql)
                .to_string(),
            [
                r#"SELECT `self_join`.`uuid` AS `A_uuid`, `self_join`.`uuid_ref` AS `A_uuid_ref`, `self_join`.`time` AS `A_time`,"#,
                r#"`r0`.`uuid` AS `B_uuid`, `r0`.`uuid_ref` AS `B_uuid_ref`, `r0`.`time` AS `B_time`"#,
                r#"FROM `self_join`"#,
                r#"LEFT JOIN `self_join` AS `r0` ON `self_join`.`uuid_ref` = `r0`.`uuid`"#,
            ]
            .join(" ")
        );
    }
}
