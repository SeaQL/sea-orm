use std::collections::HashSet;
#[cfg(not(feature = "with-time"))]
use std::time::SystemTime;

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

    if db_backend == DbBackend::Sqlite {
        db.execute_raw(Statement::from_string(
            db_backend,
            "PRAGMA foreign_keys = OFF".to_owned(),
        ))
        .await?;
    }

    if db_backend == DbBackend::MySql {
        let stmt = query_mysql_foreign_keys(db);
        let rows = db.query_all(&stmt).await?;
        for row in rows.into_iter() {
            let constraint_name: String = row.try_get("", "CONSTRAINT_NAME")?;
            let table_name: String = row.try_get("", "TABLE_NAME")?;
            let mut stmt = ForeignKey::drop();
            stmt.table(Alias::new(table_name.as_str()))
                .name(constraint_name.as_str());
            db.execute(&stmt).await?;
        }
    }

    let stmt = query_tables(db)?;
    let rows = db.query_all(&stmt).await?;
    for row in rows.into_iter() {
        let table_name: String = row.try_get("", "table_name")?;
        let mut stmt = Table::drop();
        stmt.table(Alias::new(table_name.as_str()))
            .if_exists()
            .cascade();
        db.execute(&stmt).await?;
    }

    if db_backend == DbBackend::Postgres {
        let stmt = query_pg_types(db);
        let rows = db.query_all(&stmt).await?;
        for row in rows {
            let type_name: String = row.try_get("", "typname")?;
            let mut stmt = Type::drop();
            stmt.name(Alias::new(&type_name));
            db.execute(&stmt).await?;
        }
    }

    if db_backend == DbBackend::Sqlite {
        db.execute_raw(Statement::from_string(
            db_backend,
            "PRAGMA foreign_keys = ON".to_owned(),
        ))
        .await?;
    }

    Ok(())
}

fn should_use_transaction(migration: &dyn crate::MigrationTrait, backend: DbBackend) -> bool {
    match migration.use_transaction() {
        Some(v) => v,
        None => backend == DbBackend::Postgres,
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
    pending_migrations: Vec<Migration>,
    migration_table_name: DynIden,
) -> Result<Vec<String>, DbErr> {
    let db = manager.get_connection();
    let mut applied = Vec::new();

    for Migration { migration, .. } in pending_migrations {
        if let Some(steps) = steps.as_mut() {
            if steps == &0 {
                break;
            }
            *steps -= 1;
        }

        let use_txn = should_use_transaction(migration.as_ref(), db.get_database_backend());

        if use_txn {
            let transaction = db.begin().await?;
            let txn_manager = SchemaManager::new(&transaction);
            migration.up(&txn_manager).await?;
            insert_migration_record(&transaction, migration.name(), migration_table_name.clone())
                .await?;
            transaction.commit().await?;
        } else {
            migration.up(manager).await?;
            insert_migration_record(db, migration.name(), migration_table_name.clone()).await?;
        }
        applied.push(migration.name().to_owned());
    }

    Ok(applied)
}

pub async fn exec_down_with(
    manager: &SchemaManager<'_>,
    mut steps: Option<u32>,
    applied_migrations: Vec<Migration>,
    migration_table_name: DynIden,
) -> Result<Vec<String>, DbErr> {
    let db = manager.get_connection();
    let mut rolled_back = Vec::new();

    for Migration { migration, .. } in applied_migrations.into_iter().rev() {
        if let Some(steps) = steps.as_mut() {
            if steps == &0 {
                break;
            }
            *steps -= 1;
        }

        let use_txn = should_use_transaction(migration.as_ref(), db.get_database_backend());

        if use_txn {
            let transaction = db.begin().await?;
            let txn_manager = SchemaManager::new(&transaction);
            migration.down(&txn_manager).await?;
            delete_migration_record(&transaction, migration.name(), migration_table_name.clone())
                .await?;
            transaction.commit().await?;
        } else {
            migration.down(manager).await?;
            delete_migration_record(db, migration.name(), migration_table_name.clone()).await?;
        }
        rolled_back.push(migration.name().to_owned());
    }

    Ok(rolled_back)
}
