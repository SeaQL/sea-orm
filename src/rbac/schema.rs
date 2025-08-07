use super::{AccessType, SchemaOper, entity};
use crate::{ConnectionTrait, DbConn, DbErr, EntityTrait, ExecResult, Schema};

pub async fn create_tables(db: &DbConn) -> Result<(), DbErr> {
    create_table(db, entity::permission::Entity).await?;
    create_table(db, entity::resource::Entity).await?;
    create_table(db, entity::role::Entity).await?;
    create_table(db, entity::role_hierarchy::Entity).await?;
    create_table(db, entity::role_permission::Entity).await?;
    create_table(db, entity::user_override::Entity).await?;
    create_table(db, entity::user_role::Entity).await?;

    Ok(())
}

async fn create_table<E>(db: &DbConn, entity: E) -> Result<ExecResult, DbErr>
where
    E: EntityTrait,
{
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    let res = db.execute(&schema.create_table_from_entity(entity)).await?;

    for stmt in schema.create_index_from_entity(entity) {
        db.execute(&stmt).await?;
    }

    Ok(res)
}

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
