use super::entity;
use crate::{ConnectionTrait, DbErr, EntityTrait, ExecResult, RelationDef, Schema};

#[derive(Debug, Default)]
pub struct RbacCreateTablesParams {
    pub user_override_relation: Option<RelationDef>,
    pub user_role_relation: Option<RelationDef>,
}

/// Create RBAC tables, will currently fail if any of them already exsits
pub fn create_tables<C: ConnectionTrait>(
    db: &C,
    RbacCreateTablesParams {
        user_override_relation,
        user_role_relation,
    }: RbacCreateTablesParams,
) -> Result<(), DbErr> {
    create_table(db, entity::permission::Entity, None)?;
    create_table(db, entity::resource::Entity, None)?;
    create_table(db, entity::role::Entity, None)?;
    create_table(db, entity::role_hierarchy::Entity, None)?;
    create_table(db, entity::role_permission::Entity, None)?;
    create_table(db, entity::user_override::Entity, user_override_relation)?;
    create_table(db, entity::user_role::Entity, user_role_relation)?;

    Ok(())
}

/// All tables associated with RBAC, created by SeaORM
pub fn all_tables() -> Vec<&'static str> {
    use crate::EntityName;

    vec![
        entity::permission::Entity.table_name(),
        entity::resource::Entity.table_name(),
        entity::role::Entity.table_name(),
        entity::role_hierarchy::Entity.table_name(),
        entity::role_permission::Entity.table_name(),
        entity::user_override::Entity.table_name(),
        entity::user_role::Entity.table_name(),
    ]
}

fn create_table<C, E>(db: &C, entity: E, rel: Option<RelationDef>) -> Result<ExecResult, DbErr>
where
    C: ConnectionTrait,
    E: EntityTrait,
{
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    let mut stmt = schema.create_table_from_entity(entity);
    if let Some(rel) = rel {
        stmt.foreign_key(&mut rel.into());
    }
    let res = db.execute(&stmt)?;

    for stmt in schema.create_index_from_entity(entity) {
        db.execute(&stmt)?;
    }

    Ok(res)
}
