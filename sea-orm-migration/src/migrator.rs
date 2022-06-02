use std::fmt::Display;
use std::time::SystemTime;
use tracing::info;

use sea_orm::sea_query::{Alias, Expr, ForeignKey, Query, SelectStatement, SimpleExpr, Table};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbBackend, DbConn,
    DbErr, EntityTrait, QueryFilter, QueryOrder, Schema, Statement,
};
use sea_schema::{mysql::MySql, postgres::Postgres, probe::SchemaProbe, sqlite::Sqlite};

use super::{seaql_migrations, MigrationTrait, SchemaManager};

#[derive(Debug, PartialEq)]
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
    async fn get_migration_models(db: &DbConn) -> Result<Vec<seaql_migrations::Model>, DbErr> {
        Self::install(db).await?;
        seaql_migrations::Entity::find()
            .order_by_asc(seaql_migrations::Column::Version)
            .all(db)
            .await
    }

    /// Get list of migrations with status
    async fn get_migration_with_status(db: &DbConn) -> Result<Vec<Migration>, DbErr> {
        Self::install(db).await?;
        let mut migration_files = Self::get_migration_files();
        let migration_models = Self::get_migration_models(db).await?;
        for (i, migration_model) in migration_models.into_iter().enumerate() {
            if let Some(migration_file) = migration_files.get_mut(i) {
                if migration_file.migration.name() == migration_model.version.as_str() {
                    migration_file.status = MigrationStatus::Applied;
                } else {
                    return Err(DbErr::Custom(format!("Migration mismatch: applied migration != migration file, '{0}' != '{1}'\nMigration '{0}' has been applied but its corresponding migration file is missing.", migration_file.migration.name(), migration_model.version)));
                }
            } else {
                return Err(DbErr::Custom(format!("Migration file of version '{}' is missing, this migration has been applied but its file is missing", migration_model.version)));
            }
        }
        Ok(migration_files)
    }

    /// Get list of pending migrations
    async fn get_pending_migrations(db: &DbConn) -> Result<Vec<Migration>, DbErr> {
        Self::install(db).await?;
        Ok(Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Pending)
            .collect())
    }

    /// Get list of applied migrations
    async fn get_applied_migrations(db: &DbConn) -> Result<Vec<Migration>, DbErr> {
        Self::install(db).await?;
        Ok(Self::get_migration_with_status(db)
            .await?
            .into_iter()
            .filter(|file| file.status == MigrationStatus::Applied)
            .collect())
    }

    /// Create migration table `seaql_migrations` in the database
    async fn install(db: &DbConn) -> Result<(), DbErr> {
        let builder = db.get_database_backend();
        let schema = Schema::new(builder);
        let mut stmt = schema.create_table_from_entity(seaql_migrations::Entity);
        stmt.if_not_exists();
        db.execute(builder.build(&stmt)).await.map(|_| ())
    }

    /// Drop all tables from the database, then reapply all migrations
    async fn fresh(db: &DbConn) -> Result<(), DbErr> {
        Self::install(db).await?;
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
            let mut stmt = Query::select();
            stmt.columns([Alias::new("TABLE_NAME"), Alias::new("CONSTRAINT_NAME")])
                .from((
                    Alias::new("information_schema"),
                    Alias::new("table_constraints"),
                ))
                .cond_where(
                    Condition::all()
                        .add(
                            Expr::expr(get_current_schema(db)).equals(
                                Alias::new("table_constraints"),
                                Alias::new("table_schema"),
                            ),
                        )
                        .add(Expr::expr(Expr::value("FOREIGN KEY")).equals(
                            Alias::new("table_constraints"),
                            Alias::new("constraint_type"),
                        )),
                );
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
        let stmt = query_tables(db);
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
    async fn refresh(db: &DbConn) -> Result<(), DbErr> {
        Self::down(db, None).await?;
        Self::up(db, None).await
    }

    /// Rollback all applied migrations
    async fn reset(db: &DbConn) -> Result<(), DbErr> {
        Self::down(db, None).await
    }

    /// Check the status of all migrations
    async fn status(db: &DbConn) -> Result<(), DbErr> {
        Self::install(db).await?;

        info!("Checking migration status");

        for Migration { migration, status } in Self::get_migration_with_status(db).await? {
            info!("Migration '{}'... {}", migration.name(), status);
        }

        Ok(())
    }

    /// Apply pending migrations
    async fn up(db: &DbConn, mut steps: Option<u32>) -> Result<(), DbErr> {
        Self::install(db).await?;
        let manager = SchemaManager::new(db);

        if let Some(steps) = steps {
            info!("Applying {} pending migrations", steps);
        } else {
            info!("Applying all pending migrations");
        }

        let migrations = Self::get_pending_migrations(db).await?.into_iter();
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
            migration.up(&manager).await?;
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

    /// Rollback applied migrations
    async fn down(db: &DbConn, mut steps: Option<u32>) -> Result<(), DbErr> {
        Self::install(db).await?;
        let manager = SchemaManager::new(db);

        if let Some(steps) = steps {
            info!("Rolling back {} applied migrations", steps);
        } else {
            info!("Rolling back all applied migrations");
        }

        let migrations = Self::get_applied_migrations(db).await?.into_iter().rev();
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
            migration.down(&manager).await?;
            info!("Migration '{}' has been rollbacked", migration.name());
            seaql_migrations::Entity::delete_many()
                .filter(seaql_migrations::Column::Version.eq(migration.name()))
                .exec(db)
                .await?;
        }

        Ok(())
    }

    /// Apply or Rollback migrations to version
    async fn change_to_version(db: &DbConn, version: &str) -> Result<(), DbErr> {
        let mut steps = Self::get_steps(Self::get_pending_migrations(db).await?, version);
        if steps > 0 {
            Self::up(db, Some(steps)).await
        } else {
            let mut migrations = Self::get_applied_migrations(db).await?;
            migrations.reverse();
            steps = Self::get_steps(migrations, version);
            if steps > 1 {
                Self::down(db, Some(steps - 1)).await
            } else {
                Ok(())
            }
        }
    }

    /// Apply migrations to version
    async fn up_to_version(db: &DbConn, version: &str) -> Result<(), DbErr> {
        let steps = Self::get_steps(Self::get_pending_migrations(db).await?, version);
        if steps > 0 {
            Self::up(db, Some(steps)).await
        } else {
            Ok(())
        }
    }

    /// Rollback migrations to version
    async fn down_to_version(db: &DbConn, version: &str) -> Result<(), DbErr> {
        let mut migrations = Self::get_applied_migrations(db).await?;
        migrations.reverse();
        let steps = Self::get_steps(migrations, version);
        if steps > 1 {
            Self::down(db, Some(steps - 1)).await
        } else {
            Ok(())
        }
    }

    fn get_steps(migrations: Vec<Migration>, version: &str) -> u32 {
        let mut index = 0;
        let mut matched = false;
        for Migration { migration, .. } in migrations {
            index += 1;
            if migration.name() == version {
                matched = true;
                break;
            }
        };
        if matched {
            index
        } else {
            0
        }
    }
}

pub(crate) fn query_tables(db: &DbConn) -> SelectStatement {
    match db.get_database_backend() {
        DbBackend::MySql => MySql::query_tables(),
        DbBackend::Postgres => Postgres::query_tables(),
        DbBackend::Sqlite => Sqlite::query_tables(),
    }
}

pub(crate) fn get_current_schema(db: &DbConn) -> SimpleExpr {
    match db.get_database_backend() {
        DbBackend::MySql => MySql::get_current_schema(),
        DbBackend::Postgres => Postgres::get_current_schema(),
        DbBackend::Sqlite => unimplemented!(),
    }
}

