use std::collections::HashSet;
#[cfg(not(feature = "with-time"))]
use std::time::SystemTime;
use tracing::info;

use super::{Migration, MigrationStatus, queries::*};
use crate::{SchemaManager, seaql_migrations};
use sea_orm::sea_query::{
    Alias, Expr, ExprTrait, ForeignKey, IntoIden, Order, Query, Table, extension::postgres::Type,
};
use sea_orm::{
    ActiveValue, ConnectionTrait, DbBackend, DbErr, DynIden, EntityTrait, FromQueryResult,
    Iterable, QueryFilter, Schema, Statement, TransactionSession, TransactionTrait,
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
        .take();

    db.query_all(&stmt)
        .await?
        .into_iter()
        .map(|row| seaql_migrations::Model::from_query_result(&row, ""))
        .collect()
}

pub async fn get_migration_models_read_only<C>(
    db: &C,
    migration_table_name: DynIden,
) -> Result<Vec<seaql_migrations::Model>, DbErr>
where
    C: ConnectionTrait,
{
    // Unlike `get_migration_models`, this never issues `CREATE TABLE` (see `install`), so it
    // works for a database user without DDL privileges. Existence is probed with a read-only
    // information-schema query; a missing table means nothing has been recorded yet.
    if !crate::manager::has_table(db, migration_table_name.to_string()).await? {
        return Ok(Vec::new());
    }
    get_migration_models(db, migration_table_name).await
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

pub async fn install<C>(db: &C, migration_table_name: DynIden) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let mut stmt = schema
        .create_table_from_entity(seaql_migrations::Entity)
        .table_name(migration_table_name.clone());
    stmt.if_not_exists();
    if builder == DbBackend::Postgres {
        // `IF NOT EXISTS` does not hold against another connection creating the table at the same
        // moment. One simple query runs as one transaction, so the lock is held until it is created.
        let create = builder.build(&stmt).sql;
        let lock = advisory_lock_sql(&migration_table_name);
        db.execute_unprepared(&format!("{lock}; {create}")).await?;
    } else {
        db.execute(&stmt).await?;
    }
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

pub async fn drop_everything<C: ConnectionTrait + TransactionTrait>(db: &C) -> Result<(), DbErr> {
    if db.get_database_backend() == DbBackend::Postgres {
        let transaction = db.begin().await?;
        drop_everything_impl(&transaction).await?;
        transaction.commit().await
    } else {
        drop_everything_impl(db).await
    }
}

async fn drop_everything_impl<C: ConnectionTrait>(db: &C) -> Result<(), DbErr> {
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
            stmt.name(Alias::new(&type_name)).if_exists().cascade();
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

fn should_use_transaction(migration: &dyn crate::MigrationTrait, backend: DbBackend) -> bool {
    match migration.use_transaction() {
        Some(v) => v,
        None => match backend {
            DbBackend::Postgres => true,
            DbBackend::Sqlite => cfg!(feature = "feat-sqlite-transactional-migrations"),
            _ => false,
        },
    }
}

/// A Postgres lock, held until the end of the transaction, on the migration table of the current
/// schema.
fn advisory_lock_sql(migration_table_name: &DynIden) -> String {
    let key = migration_table_name.to_string().replace('\'', "''");
    format!(
        "SELECT pg_advisory_xact_lock(hashtext('sea-orm-migration:' || current_schema() || '.{key}'))"
    )
}

/// On Postgres, serializes the migration's transaction against other processes running a migrator
/// on the same table, and returns the applied versions, read after the lock is taken. Other
/// backends and migrations run outside of a transaction are not locked.
async fn lock_and_read_applied<C: ConnectionTrait>(
    db: &C,
    migration_table_name: DynIden,
) -> Result<Option<HashSet<String>>, DbErr> {
    if db.get_database_backend() != DbBackend::Postgres {
        return Ok(None);
    }
    db.execute_unprepared(&advisory_lock_sql(&migration_table_name))
        .await?;
    let models = get_migration_models(db, migration_table_name).await?;
    Ok(Some(
        models.into_iter().map(|model| model.version).collect(),
    ))
}

/// Whether the `i`-th migration still has to be applied, given the versions applied right now:
/// `false` if another process applied it, an error if it rolled back an earlier one.
fn should_apply(names: &[String], i: usize, applied: &HashSet<String>) -> Result<bool, DbErr> {
    if applied.contains(&names[i]) {
        return Ok(false);
    }
    match names[..i].iter().find(|name| !applied.contains(*name)) {
        Some(name) => Err(DbErr::Migration(format!(
            "Migration '{name}' has been rolled back by another process"
        ))),
        None => Ok(true),
    }
}

/// Whether the `i`-th migration still has to be rolled back, given the versions applied right
/// now: `false` if another process rolled it back, an error if it applied a later one.
fn should_roll_back(names: &[String], i: usize, applied: &HashSet<String>) -> Result<bool, DbErr> {
    if !applied.contains(&names[i]) {
        return Ok(false);
    }
    match names[i + 1..].iter().find(|name| applied.contains(*name)) {
        Some(name) => Err(DbErr::Migration(format!(
            "Migration '{name}' has been applied by another process"
        ))),
        None => Ok(true),
    }
}

async fn insert_migration_record<C: ConnectionTrait>(
    db: &C,
    name: &str,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    #[cfg(not(feature = "with-time"))]
    let applied_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_secs() as i64;
    #[cfg(feature = "with-time")]
    let applied_at = sea_orm::prelude::TimeDateTimeWithTimeZone::now_utc().unix_timestamp();
    seaql_migrations::Entity::insert(seaql_migrations::ActiveModel {
        version: ActiveValue::Set(name.to_owned()),
        applied_at: ActiveValue::Set(applied_at),
    })
    .table_name(migration_table_name)
    .exec(db)
    .await?;
    Ok(())
}

async fn delete_migration_record<C: ConnectionTrait>(
    db: &C,
    name: &str,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    seaql_migrations::Entity::delete_many()
        .filter(Expr::col(seaql_migrations::Column::Version).eq(name))
        .table_name(migration_table_name)
        .exec(db)
        .await?;
    Ok(())
}

pub async fn exec_up_with(
    manager: &SchemaManager<'_>,
    mut steps: Option<u32>,
    migrations: Vec<Migration>,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let names: Vec<String> = migrations.iter().map(|m| m.name().to_owned()).collect();
    let pending_migrations: Vec<_> = migrations
        .into_iter()
        .enumerate()
        .filter(|(_, m)| m.status == MigrationStatus::Pending)
        .collect();

    if let Some(steps) = steps {
        info!("Applying {} pending migrations", steps);
    } else {
        info!("Applying all pending migrations");
    }
    if pending_migrations.is_empty() {
        info!("No pending migrations");
    }

    for (i, Migration { migration, .. }) in pending_migrations {
        if let Some(steps) = steps.as_mut() {
            if steps == &0 {
                break;
            }
            *steps -= 1;
        }

        let use_txn = should_use_transaction(migration.as_ref(), db.get_database_backend());
        info!("Applying migration '{}'", migration.name());

        if use_txn {
            let transaction = db.begin().await?;
            let applied = lock_and_read_applied(&transaction, migration_table_name.clone()).await?;
            if !applied.map_or(Ok(true), |applied| should_apply(&names, i, &applied))? {
                info!(
                    "Migration '{}' has been applied by another process",
                    migration.name()
                );
                continue;
            }
            let txn_manager = SchemaManager::new(&transaction);
            migration.up(&txn_manager).await?;
            info!("Migration '{}' has been applied", migration.name());
            insert_migration_record(&transaction, migration.name(), migration_table_name.clone())
                .await?;
            transaction.commit().await?;
        } else {
            migration.up(manager).await?;
            info!("Migration '{}' has been applied", migration.name());
            insert_migration_record(db, migration.name(), migration_table_name.clone()).await?;
        }
    }

    Ok(())
}

pub async fn exec_down_with(
    manager: &SchemaManager<'_>,
    mut steps: Option<u32>,
    migrations: Vec<Migration>,
    migration_table_name: DynIden,
) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let names: Vec<String> = migrations.iter().map(|m| m.name().to_owned()).collect();
    let applied_migrations: Vec<_> = migrations
        .into_iter()
        .enumerate()
        .filter(|(_, m)| m.status == MigrationStatus::Applied)
        .collect();

    if let Some(steps) = steps {
        info!("Rolling back {} applied migrations", steps);
    } else {
        info!("Rolling back all applied migrations");
    }
    if applied_migrations.is_empty() {
        info!("No applied migrations");
    }

    for (i, Migration { migration, .. }) in applied_migrations.into_iter().rev() {
        if let Some(steps) = steps.as_mut() {
            if steps == &0 {
                break;
            }
            *steps -= 1;
        }

        let use_txn = should_use_transaction(migration.as_ref(), db.get_database_backend());
        info!("Rolling back migration '{}'", migration.name());

        if use_txn {
            let transaction = db.begin().await?;
            let applied = lock_and_read_applied(&transaction, migration_table_name.clone()).await?;
            if !applied.map_or(Ok(true), |applied| should_roll_back(&names, i, &applied))? {
                info!(
                    "Migration '{}' has been rolled back by another process",
                    migration.name()
                );
                continue;
            }
            let txn_manager = SchemaManager::new(&transaction);
            migration.down(&txn_manager).await?;
            info!("Migration '{}' has been rolled back", migration.name());
            delete_migration_record(&transaction, migration.name(), migration_table_name.clone())
                .await?;
            transaction.commit().await?;
        } else {
            migration.down(manager).await?;
            info!("Migration '{}' has been rolled back", migration.name());
            delete_migration_record(db, migration.name(), migration_table_name.clone()).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        ["m1", "m2", "m3"].map(String::from).to_vec()
    }

    fn applied(versions: &[&str]) -> HashSet<String> {
        versions.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn apply_after_another_process() {
        assert!(should_apply(&names(), 1, &applied(&["m1"])).unwrap());
        // Another `up` applied it first.
        assert!(!should_apply(&names(), 1, &applied(&["m1", "m2"])).unwrap());
        // A concurrent `down` rolled back the migration it builds on.
        let err = should_apply(&names(), 1, &applied(&[])).unwrap_err();
        assert!(
            err.to_string().contains("'m1' has been rolled back"),
            "{err}"
        );
    }

    #[test]
    fn roll_back_after_another_process() {
        assert!(should_roll_back(&names(), 1, &applied(&["m1", "m2"])).unwrap());
        // Another `down` rolled it back first.
        assert!(!should_roll_back(&names(), 1, &applied(&["m1"])).unwrap());
        // A concurrent `up` applied a migration that builds on it.
        let err = should_roll_back(&names(), 1, &applied(&["m1", "m2", "m3"])).unwrap_err();
        assert!(err.to_string().contains("'m3' has been applied"), "{err}");
    }
}
