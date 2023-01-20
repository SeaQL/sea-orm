use std::collections::HashSet;
use std::fmt::Display;
use std::time::SystemTime;
use tracing::info;

use sea_orm::sea_query::{
    self, extension::postgres::Type, Alias, Expr, ForeignKey, Iden, JoinType, Query,
    SelectStatement, SimpleExpr, Table,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbBackend, DbErr,
    EntityTrait, QueryFilter, QueryOrder, Schema, Statement,
};
use sea_schema::{mysql::MySql, postgres::Postgres, probe::SchemaProbe, sqlite::Sqlite};

use super::{seaql_migrations, IntoSchemaManagerConnection, MigrationTrait, SchemaManager};

#[derive(Debug, PartialEq, Eq)]
/// Status of migration
pub enum MigrationStatus {
    /// Not yet applied
    Pending,
    /// Applied
    Applied,
}

pub struct Migration {
    migration: Box<dyn MigrationTrait>,
    status: MigrationStatus,
}

impl Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            MigrationStatus::Pending => "Pending",
            MigrationStatus::Applied => "Applied",
        };
        write!(f, "{}", status)
    }
}

/// Performing migrations on a database
#[async_trait::async_trait]
pub trait MigratorTrait: Send {
    /// Vector of migrations in time sequence
    fn migrations() -> Vec<Box<dyn MigrationTrait>>;

    /// Get list of migrations wrapped in `Migration` struct
    fn get_migration_files() -> Vec<Migration> {
        Self::migrations()
            .into_iter()
            .map(|migration| Migration {
                migration,
                status: MigrationStatus::Pending,
            })
            .collect()
    }

    /// Get list of applied migrations from database
    async fn get_migration_models<C>(db: &C) -> Result<Vec<seaql_migrations::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        seaql_migrations::Entity::find()
            .order_by_asc(seaql_migrations::Column::Version)
            .all(db)
            .await
    }

    /// Get list of migrations with status
    async fn get_migration_with_status<C>(db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        let mut migration_files = Self::get_migration_files();
        let migration_models = Self::get_migration_models(db).await?;

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
                format!("Migration file of version '{}' is missing, this migration has been applied but its file is missing", missing_migration)
            }).collect();

        if !errors.is_empty() {
            Err(DbErr::Custom(errors.join("\n")))
        } else {
            Ok(migration_files)
        }
    }

    /// Get list of pending migrations
    async fn get_pending_migrations<C>(db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        Ok(Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Pending)
            .collect())
    }

    /// Get list of applied migrations
    async fn get_applied_migrations<C>(db: &C) -> Result<Vec<Migration>, DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;
        Ok(Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Applied)
            .collect())
    }

    /// Create migration table `seaql_migrations` in the database
    async fn install<C>(db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        let schema = Schema::new(builder);
        let mut stmt = schema.create_table_from_entity(seaql_migrations::Entity);
        stmt.if_not_exists();
        db.execute(builder.build(&stmt)).await.map(|_| ())
    }

    /// Drop all tables from the database, then reapply all migrations
    async fn fresh<'c, C>(db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let db = db.into_schema_manager_connection();

        Self::install(&db).await?;
        let db_backend = db.get_database_backend();

        // Temporarily disable the foreign key check
        if db_backend == DbBackend::Sqlite {
            info!("Disabling foreign key check");
            db.execute(Statement::from_string(
                db_backend,
                "PRAGMA foreign_keys = OFF".to_owned(),
            ))
            .await?;
            info!("Foreign key check disabled");
        }

        // Drop all foreign keys
        if db_backend == DbBackend::MySql {
            info!("Dropping all foreign keys");
            let stmt = query_mysql_foreign_keys(&db);
            let rows = db.query_all(db_backend.build(&stmt)).await?;
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
                db.execute(db_backend.build(&stmt)).await?;
                info!("Foreign key '{}' has been dropped", constraint_name);
            }
            info!("All foreign keys dropped");
        }

        // Drop all tables
        let stmt = query_tables(&db);
        let rows = db.query_all(db_backend.build(&stmt)).await?;
        for row in rows.into_iter() {
            let table_name: String = row.try_get("", "table_name")?;
            info!("Dropping table '{}'", table_name);
            let mut stmt = Table::drop();
            stmt.table(Alias::new(table_name.as_str()))
                .if_exists()
                .cascade();
            db.execute(db_backend.build(&stmt)).await?;
            info!("Table '{}' has been dropped", table_name);
        }

        // Drop all types
        if db_backend == DbBackend::Postgres {
            info!("Dropping all types");
            let stmt = query_pg_types(&db);
            let rows = db.query_all(db_backend.build(&stmt)).await?;
            for row in rows {
                let type_name: String = row.try_get("", "typname")?;
                info!("Dropping type '{}'", type_name);
                let mut stmt = Type::drop();
                stmt.name(Alias::new(&type_name as &str));
                db.execute(db_backend.build(&stmt)).await?;
                info!("Type '{}' has been dropped", type_name);
            }
        }

        // Restore the foreign key check
        if db_backend == DbBackend::Sqlite {
            info!("Restoring foreign key check");
            db.execute(Statement::from_string(
                db_backend,
                "PRAGMA foreign_keys = ON".to_owned(),
            ))
            .await?;
            info!("Foreign key check restored");
        }

        // Reapply all migrations
        Self::up(db, None).await
    }

    /// Rollback all applied migrations, then reapply all migrations
    async fn refresh<'c, C>(db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let manager = SchemaManager::new(db);
        down_internal::<Self>(&manager, None).await?;
        up_internal::<Self>(&manager, None).await
    }

    /// Rollback all applied migrations
    async fn reset<'c, C>(db: C) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let manager = SchemaManager::new(db);
        down_internal::<Self>(&manager, None).await
    }

    /// Check the status of all migrations
    async fn status<C>(db: &C) -> Result<(), DbErr>
    where
        C: ConnectionTrait,
    {
        Self::install(db).await?;

        info!("Checking migration status");

        for Migration { migration, status } in Self::get_migration_with_status(db).await? {
            info!("Migration '{}'... {}", migration.name(), status);
        }

        Ok(())
    }

    /// Apply pending migrations
    async fn up<'c, C>(db: C, steps: Option<u32>) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let manager = SchemaManager::new(db);
        up_internal::<Self>(&manager, steps).await
    }

    /// Rollback applied migrations
    async fn down<'c, C>(db: C, steps: Option<u32>) -> Result<(), DbErr>
    where
        C: IntoSchemaManagerConnection<'c>,
    {
        let manager = SchemaManager::new(db);
        down_internal::<Self>(&manager, steps).await
    }
}

async fn up_internal<M>(manager: &SchemaManager<'_>, mut steps: Option<u32>) -> Result<(), DbErr>
where
    M: MigratorTrait + ?Sized,
{
    let db = manager.get_connection();

    M::install(db).await?;

    if let Some(steps) = steps {
        info!("Applying {} pending migrations", steps);
    } else {
        info!("Applying all pending migrations");
    }

    let migrations = M::get_pending_migrations(db).await?.into_iter();
    if migrations.len() == 0 {
        info!("No pending migrations");
    }
    for Migration { migration, .. } in migrations {
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
        seaql_migrations::ActiveModel {
            version: ActiveValue::Set(migration.name().to_owned()),
            applied_at: ActiveValue::Set(now.as_secs() as i64),
        }
        .insert(db)
        .await?;
    }

    Ok(())
}

async fn down_internal<M>(manager: &SchemaManager<'_>, mut steps: Option<u32>) -> Result<(), DbErr>
where
    M: MigratorTrait + ?Sized,
{
    let db = manager.get_connection();

    M::install(db).await?;

    if let Some(steps) = steps {
        info!("Rolling back {} applied migrations", steps);
    } else {
        info!("Rolling back all applied migrations");
    }

    let migrations = M::get_applied_migrations(db).await?.into_iter().rev();
    if migrations.len() == 0 {
        info!("No applied migrations");
    }
    for Migration { migration, .. } in migrations {
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
            .filter(seaql_migrations::Column::Version.eq(migration.name()))
            .exec(db)
            .await?;
    }

    Ok(())
}

fn query_tables<C>(db: &C) -> SelectStatement
where
    C: ConnectionTrait,
{
    match db.get_database_backend() {
        DbBackend::MySql => MySql::query_tables(),
        DbBackend::Postgres => Postgres::query_tables(),
        DbBackend::Sqlite => Sqlite::query_tables(),
    }
}

fn get_current_schema<C>(db: &C) -> SimpleExpr
where
    C: ConnectionTrait,
{
    match db.get_database_backend() {
        DbBackend::MySql => MySql::get_current_schema(),
        DbBackend::Postgres => Postgres::get_current_schema(),
        DbBackend::Sqlite => unimplemented!(),
    }
}

#[derive(Iden)]
enum InformationSchema {
    #[iden = "information_schema"]
    Schema,
    #[iden = "TABLE_NAME"]
    TableName,
    #[iden = "CONSTRAINT_NAME"]
    ConstraintName,
    TableConstraints,
    TableSchema,
    ConstraintType,
}

fn query_mysql_foreign_keys<C>(db: &C) -> SelectStatement
where
    C: ConnectionTrait,
{
    let mut stmt = Query::select();
    stmt.columns([
        InformationSchema::TableName,
        InformationSchema::ConstraintName,
    ])
    .from((
        InformationSchema::Schema,
        InformationSchema::TableConstraints,
    ))
    .cond_where(
        Condition::all()
            .add(Expr::expr(get_current_schema(db)).equals((
                InformationSchema::TableConstraints,
                InformationSchema::TableSchema,
            )))
            .add(
                Expr::col((
                    InformationSchema::TableConstraints,
                    InformationSchema::ConstraintType,
                ))
                .eq("FOREIGN KEY"),
            ),
    );
    stmt
}

#[derive(Iden)]
enum PgType {
    Table,
    Typname,
    Typnamespace,
    Typelem,
}

#[derive(Iden)]
enum PgNamespace {
    Table,
    Oid,
    Nspname,
}

fn query_pg_types<C>(db: &C) -> SelectStatement
where
    C: ConnectionTrait,
{
    let mut stmt = Query::select();
    stmt.column(PgType::Typname)
        .from(PgType::Table)
        .join(
            JoinType::LeftJoin,
            PgNamespace::Table,
            Expr::col((PgNamespace::Table, PgNamespace::Oid))
                .equals((PgType::Table, PgType::Typnamespace)),
        )
        .cond_where(
            Condition::all()
                .add(
                    Expr::expr(get_current_schema(db))
                        .equals((PgNamespace::Table, PgNamespace::Nspname)),
                )
                .add(Expr::col((PgType::Table, PgType::Typelem)).eq(0)),
        );
    stmt
}
