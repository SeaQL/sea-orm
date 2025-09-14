use super::{AccessType, SchemaOper, entity};
use crate::{ConnectionTrait, DbConn, DbErr, EntityTrait, ExecResult, RelationDef, Schema};

#[derive(Debug, Default)]
pub struct RbacCreateTablesParams {
    user_override_relation: Option<RelationDef>,
    user_role_relation: Option<RelationDef>,
}

/// Create RBAC tables, will currently fail if any of them already exsits
pub async fn create_tables(
    db: &DbConn,
    RbacCreateTablesParams {
        user_override_relation,
        user_role_relation,
    }: RbacCreateTablesParams,
) -> Result<(), DbErr> {
    create_table(db, entity::permission::Entity, None).await?;
    create_table(db, entity::resource::Entity, None).await?;
    create_table(db, entity::role::Entity, None).await?;
    create_table(db, entity::role_hierarchy::Entity, None).await?;
    create_table(db, entity::role_permission::Entity, None).await?;
    create_table(db, entity::user_override::Entity, user_override_relation).await?;
    create_table(db, entity::user_role::Entity, user_role_relation).await?;

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

async fn create_table<E>(
    db: &DbConn,
    entity: E,
    rel: Option<RelationDef>,
) -> Result<ExecResult, DbErr>
where
    E: EntityTrait,
{
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    let mut stmt = schema.create_table_from_entity(entity);
    if let Some(rel) = rel {
        stmt.foreign_key(&mut rel.into());
    }
    let res = db.execute(&stmt).await?;

    for stmt in schema.create_index_from_entity(entity) {
        db.execute(&stmt).await?;
    }

    Ok(res)
}

/// Mapping of AccessType to &str
pub fn action_str(at: &AccessType) -> &'static str {
    match at {
        AccessType::Select => "select",
        AccessType::Insert => "insert",
        AccessType::Update => "update",
        AccessType::Delete => "delete",
        AccessType::Schema(SchemaOper::Create) => "schema_create",
        AccessType::Schema(SchemaOper::Alter) => "schema_alter",
        AccessType::Schema(SchemaOper::Drop) => "schema_drop",
        AccessType::Schema(SchemaOper::Rename) => "schema_rename",
        AccessType::Schema(SchemaOper::Truncate) => "schema_truncate",
    }
}
