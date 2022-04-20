//! Convert database backend

use sea_schema::migration;

pub(crate) fn into_orm_db_backend(db_backend: migration::DbBackend) -> sea_orm::DbBackend {
    match db_backend {
        migration::DbBackend::MySql => sea_orm::DbBackend::MySql,
        migration::DbBackend::Postgres => sea_orm::DbBackend::Postgres,
        migration::DbBackend::Sqlite => sea_orm::DbBackend::Sqlite,
    }
}

pub(crate) fn into_migration_db_backend(db_backend: sea_orm::DbBackend) -> migration::DbBackend {
    match db_backend {
        sea_orm::DbBackend::MySql => migration::DbBackend::MySql,
        sea_orm::DbBackend::Postgres => migration::DbBackend::Postgres,
        sea_orm::DbBackend::Sqlite => migration::DbBackend::Sqlite,
    }
}
