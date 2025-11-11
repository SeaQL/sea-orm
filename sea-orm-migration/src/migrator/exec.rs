use std::collections::HashSet;
use std::time::SystemTime;
use tracing::info;

use super::{Migration, MigrationStatus, queries::*};
use crate::{SchemaManager, seaql_migrations};
use sea_orm::sea_query::{
    Alias, Expr, ExprTrait, ForeignKey, IntoIden, Order, Query, Table, extension::postgres::Type,
};
use sea_orm::{
    ActiveValue, ConnectionTrait, DbBackend, DbErr, DynIden, EntityTrait, FromQueryResult,
    Iterable, QueryFilter, Schema, Statement,
};

pub async fn get_migration_models<C>(
    db: &C,
    migration_table_name: DynIden,
) -> Result<Vec<seaql_migrations::Model>, DbErr>
where
    C: ConnectionTrait,
{
    let stmt = Query::select()
        .table_name(migration_table_name)
        .columns(seaql_migrations::Column::iter().map(IntoIden::into_iden))
        .order_by(seaql_migrations::Column::Version, Order::Asc)
        .to_owned();
    let builder = db.get_database_backend();
    seaql_migrations::Model::find_by_statement(builder.build(&stmt))
        .all(db)
        .await
}

pub fn get_migration_with_status(
    migration_files: Vec<Migration>,
    migration_models: Vec<seaql_migrations::Model>,
) -> Result<Vec<Migration>, DbErr> {
    let mut migration_files = migration_files;

    let migration_in_db: HashSet<String> = migration_models
        .into_iter()
        .map(|model| model.version)
        .collect();
    let migration_in_fs: HashSet<String> = migration_files
        .iter()
        .map(|file| file.migration.name().to_string())
        .collect();

    let pending_migrations = &migration_in_fs - &migration_in_db;
    for migration_file in migration_files.iter_mut() {
        if !pending_migrations.contains(migration_file.migration.name()) {
            migration_file.status = MigrationStatus::Applied;
        }
    }

    let missing_migrations_in_fs = &migration_in_db - &migration_in_fs;
    let errors: Vec<String> = missing_migrations_in_fs
            .iter()
            .map(|missing_migration| {
                format!("Migration file of version '{missing_migration}' is missing, this migration has been applied but its file is missing")
            }).collect();

    if !errors.is_empty() {
        Err(DbErr::Custom(errors.join("\n")))
    } else {
        Ok(migration_files)
    }
}

macro_rules! exec_with_connection {
    ($db:ident, $fn:expr) => {{
        async {
            let db = $db.into_schema_manager_connection();

            match db.get_database_backend() {
                DbBackend::Postgres => {
                    let transaction = db.begin().await?;
                    let manager = SchemaManager::new(&transaction);
                    $fn(&manager).await?;
                    transaction.commit().await
                }
                DbBackend::MySql | DbBackend::Sqlite => {
                    let manager = SchemaManager::new(db);
                    $fn(&manager).await
                }
                db => Err(DbErr::BackendNotSupported {
                    db: db.as_str(),
                    ctx: "exec_with_connection",
                }),
            }
        }
    }};
}

pub(crate) use exec_with_connection;

pub async fn install<C>(db: &C, migration_table_name: DynIden) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut stmt = schema
        .create_table_from_entity(seaql_migrations::Entity)
        .table_name(migration_table_name);
    stmt.if_not_exists();
    db.execute(&stmt).await?;
    Ok(())
}

pub async fn uninstall(
    manager: &SchemaManager<'_>,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    let mut stmt = Table::drop();
    stmt.table(migration_table_name).if_exists().cascade();
    manager.drop_table(stmt).await?;
    Ok(())
}

pub async fn drop_everything<C: ConnectionTrait>(db: &C) -> Result<(), DbErr> {
    let db_backend = db.get_database_backend();

    // Temporarily disable the foreign key check
    if db_backend == DbBackend::Sqlite {
        info!("Disabling foreign key check");
        db.execute_raw(Statement::from_string(
            db_backend,
            "PRAGMA foreign_keys = OFF".to_owned(),
        ))
        .await?;
        info!("Foreign key check disabled");
    }

    // Drop all foreign keys
    if db_backend == DbBackend::MySql {
        info!("Dropping all foreign keys");
        let stmt = query_mysql_foreign_keys(db);
        let rows = db.query_all(&stmt).await?;
        for row in rows.into_iter() {
            let constraint_name: String = row.try_get("", "CONSTRAINT_NAME")?;
            let table_name: String = row.try_get("", "TABLE_NAME")?;
            info!(
                "Dropping foreign key '{}' from table '{}'",
                constraint_name, table_name
            );
            let mut stmt = ForeignKey::drop();
            stmt.table(Alias::new(table_name.as_str()))
                .name(constraint_name.as_str());
            db.execute(&stmt).await?;
            info!("Foreign key '{}' has been dropped", constraint_name);
        }
        info!("All foreign keys dropped");
    }

    // Drop all tables
    let stmt = query_tables(db)?;
    let rows = db.query_all(&stmt).await?;
    for row in rows.into_iter() {
        let table_name: String = row.try_get("", "table_name")?;
        info!("Dropping table '{}'", table_name);
        let mut stmt = Table::drop();
        stmt.table(Alias::new(table_name.as_str()))
            .if_exists()
            .cascade();
        db.execute(&stmt).await?;
        info!("Table '{}' has been dropped", table_name);
    }

    // Drop all types
    if db_backend == DbBackend::Postgres {
        info!("Dropping all types");
        let stmt = query_pg_types(db);
        let rows = db.query_all(&stmt).await?;
        for row in rows {
            let type_name: String = row.try_get("", "typname")?;
            info!("Dropping type '{}'", type_name);
            let mut stmt = Type::drop();
            stmt.name(Alias::new(&type_name));
            db.execute(&stmt).await?;
            info!("Type '{}' has been dropped", type_name);
        }
    }

    // Restore the foreign key check
    if db_backend == DbBackend::Sqlite {
        info!("Restoring foreign key check");
        db.execute_raw(Statement::from_string(
            db_backend,
            "PRAGMA foreign_keys = ON".to_owned(),
        ))
        .await?;
        info!("Foreign key check restored");
    }

    Ok(())
}

pub async fn exec_up_with(
    manager: &SchemaManager<'_>,
    mut steps: Option<u32>,
    pending_migrations: Vec<Migration>,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    let db = manager.get_connection();

    if let Some(steps) = steps {
        info!("Applying {} pending migrations", steps);
    } else {
        info!("Applying all pending migrations");
    }
    if pending_migrations.is_empty() {
        info!("No pending migrations");
    }

    for Migration { migration, .. } in pending_migrations {
        if let Some(steps) = steps.as_mut() {
            if steps == &0 {
                break;
            }
            *steps -= 1;
        }
        info!("Applying migration '{}'", migration.name());
        migration.up(manager).await?;
        info!("Migration '{}' has been applied", migration.name());
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!");
        seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
            version: ActiveValue::Set(migration.name().to_owned()),
            applied_at: ActiveValue::Set(now.as_secs() as i64),
        })
        .table_name(migration_table_name.clone())
        .exec(db)
        .await?;
    }

    Ok(())
}

pub async fn exec_down_with(
    manager: &SchemaManager<'_>,
    mut steps: Option<u32>,
    applied_migrations: Vec<Migration>,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    let db = manager.get_connection();

    if let Some(steps) = steps {
        info!("Rolling back {} applied migrations", steps);
    } else {
        info!("Rolling back all applied migrations");
    }
    if applied_migrations.is_empty() {
        info!("No applied migrations");
    }

    for Migration { migration, .. } in applied_migrations.into_iter().rev() {
        if let Some(steps) = steps.as_mut() {
            if steps == &0 {
                break;
            }
            *steps -= 1;
        }
        info!("Rolling back migration '{}'", migration.name());
        migration.down(manager).await?;
        info!("Migration '{}' has been rollbacked", migration.name());
        seaql_migrations::Entity::delete_many()
            .filter(Expr::col(seaql_migrations::Column::Version).eq(migration.name()))
            .table_name(migration_table_name.clone())
            .exec(db)
            .await?;
    }

    Ok(())
}
