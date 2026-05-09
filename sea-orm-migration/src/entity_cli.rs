use chrono::Utc;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DbBackend, InterpretConfig, Schema, interpret_changes};

use crate::{
    EntitySet, MigratorTraitSelf,
    codegen::MigrationMetadata,
    fs::write_migration,
    response::{
        ApiMeta, ApiResponse, DiffData, GenerateData, SchemaData, SuggestionJson,
        UnresolvedRenameJson, WarningJson, fnv64_hex,
    },
    summary::summarize,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "entity",
    about = "Entity-first migration tool for SeaORM",
    version
)]
struct Cli {
    #[arg(
        global = true,
        short = 'u',
        long,
        env = "DATABASE_URL",
        help = "Database URL"
    )]
    database_url: Option<String>,

    #[arg(
        global = true,
        short = 's',
        long,
        env = "DATABASE_SCHEMA",
        long_help = "Database schema\n \
                    - For MySQL and SQLite, this argument is ignored.\n \
                    - For PostgreSQL, this argument is optional with default value 'public'.\n"
    )]
    database_schema: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Discover schema changes between entity definitions and the live database.
    ///
    /// Returns JSON with discovered SQL statements, warnings, suggestions, and
    /// any ambiguous renames that must be resolved before calling `generate`.
    /// Never writes any files.
    Diff {
        /// Allow dangerous operations (e.g. dropping tables) in the diff
        #[arg(long, default_value_t = true)]
        allow_dangerous: bool,
    },

    /// Generate a migration file from entity definitions.
    ///
    /// Requires that `diff` was run first. Pass the `schema_hash` from the diff
    /// output via `--schema-hash` so stale calls are rejected. All ambiguous
    /// renames reported by `diff` must be resolved via `--rename` flags.
    Generate {
        /// Path to the migration crate directory
        #[arg(long, default_value = "../migration")]
        migration_dir: String,

        /// Name for the migration (e.g. `add_users`)
        #[arg(required = true)]
        name: String,

        /// Schema hash from the preceding `diff` output — used to detect staleness
        #[arg(long, required = true)]
        schema_hash: String,

        #[arg(
            long,
            default_value = "true",
            help = "Generate migration file based on Utc time",
            conflicts_with = "local_time",
            display_order = 1001
        )]
        universal_time: bool,

        #[arg(
            long,
            help = "Generate migration file based on Local time",
            conflicts_with = "universal_time",
            display_order = 1002
        )]
        local_time: bool,

        /// Allow dangerous operations (must match the value used in `diff`)
        #[arg(long, default_value_t = true)]
        allow_dangerous: bool,

        /// Resolve an ambiguous rename in the format TABLE.OLD_COL:NEW_COL
        #[arg(long = "rename", value_name = "TABLE.OLD:NEW")]
        renames: Vec<String>,
    },

    /// Preview the schema as defined by the registered entities, as SQL DDL statements.
    ///
    /// Does not connect to any database. Returns a JSON object with a `statements` array
    /// of CREATE TABLE / CREATE TYPE / CREATE INDEX SQL strings, rendered for the
    /// target database backend (specified via `--database-backend`).
    Schema {
        /// Database backend to render SQL for (postgres, mysql, sqlite)
        #[arg(long, default_value = "postgres")]
        database_backend: String,
    },

    #[command(
        about = "Drop all tables from the database, then reapply all migrations",
        display_order = 30
    )]
    Fresh,
    #[command(
        about = "Rollback all applied migrations, then reapply all migrations",
        display_order = 40
    )]
    Refresh,
    #[command(about = "Rollback all applied migrations", display_order = 50)]
    Reset,
    #[command(about = "Check the status of all migrations", display_order = 60)]
    Status,
    #[command(about = "Apply pending migrations", display_order = 70)]
    Up {
        #[arg(short, long, help = "Number of pending migrations to apply")]
        num: Option<u32>,
    },
    #[command(about = "Rollback applied migrations", display_order = 80)]
    Down {
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Number of applied migrations to roll back",
            display_order = 90
        )]
        num: u32,
    },
}

/// Run the entity-first CLI with the given entity set and migrator.
///
/// Call this from your entity crate's `main.rs`:
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() {
///     sea_orm_migration::entity_cli::run_cli(Entities, migration::Migrator).await;
/// }
/// ```
pub async fn run_cli<E, M>(entity_set: E, migrator: M)
where
    E: EntitySet,
    M: MigratorTraitSelf,
{
    dotenv().ok();
    let cli = Cli::parse();

    // Handle commands that don't need a DB connection first.
    if let Some(Commands::Schema { database_backend }) = cli.command {
        let meta = build_meta(&migrator, None);
        match run_schema(entity_set, &database_backend) {
            Ok(data) => println!(
                "{}",
                serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
            ),
            Err(e) => {
                emit_err::<()>(meta, e);
                std::process::exit(1);
            }
        }
        return;
    }

    let url = cli
        .database_url
        .expect("Environment variable 'DATABASE_URL' not set");
    let schema_path = cli.database_schema.unwrap_or_else(|| "public".to_owned());

    let db = match Database::connect(
        ConnectOptions::new(url)
            .set_schema_search_path(schema_path)
            .to_owned(),
    )
    .await
    {
        Ok(db) => db,
        Err(e) => {
            let meta = build_meta(&migrator, None);
            emit_err::<()>(meta, e);
            std::process::exit(1);
        }
    };

    let migration_table = migrator.migration_table_name().to_string();

    match cli.command {
        Some(Commands::Schema { .. }) => unreachable!("handled above"),
        Some(Commands::Diff { allow_dangerous }) => {
            let meta = build_meta(&migrator, None);
            let pending = match migrator.get_pending_migrations(&db).await {
                Ok(p) => p,
                Err(e) => {
                    emit_err::<()>(meta, e);
                    std::process::exit(1);
                }
            };
            if !pending.is_empty() {
                let names: Vec<String> = pending.iter().map(|m| m.name().to_owned()).collect();
                emit_err::<()>(
                    meta,
                    format!(
                        "{} pending migration(s) must be applied before running diff:\n  {}\nRun `migrate up` first.",
                        names.len(),
                        names.join("\n  ")
                    ),
                );
                std::process::exit(1);
            }
            match run_diff(entity_set, &db, allow_dangerous, &migration_table).await {
                Ok(data) => println!(
                    "{}",
                    serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                ),
                Err(e) => {
                    emit_err::<()>(meta, e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Generate {
            migration_dir,
            name,
            schema_hash,
            local_time,
            universal_time: _,
            allow_dangerous,
            renames,
        }) => {
            let meta = build_meta(&migrator, None);
            match run_generate(
                entity_set,
                &db,
                &migration_dir,
                &name,
                &schema_hash,
                local_time,
                allow_dangerous,
                &renames,
                &migration_table,
            )
            .await
            {
                Ok(data) => println!(
                    "{}",
                    serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                ),
                Err(e) => {
                    emit_err::<()>(meta, e);
                    std::process::exit(1);
                }
            }
        }

        migrate_cmd => {
            let meta = build_meta(&migrator, None);
            match migrate_cmd {
                Some(Commands::Up { num }) => match migrator.up(&db, num).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                Some(Commands::Down { num }) => match migrator.down(&db, Some(num)).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                Some(Commands::Fresh) => match migrator.fresh(&db).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                Some(Commands::Refresh) => match migrator.refresh(&db).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                Some(Commands::Reset) => match migrator.reset(&db).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                Some(Commands::Status) => match migrator.status(&db).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                None => match migrator.up(&db, None).await {
                    Ok(data) => println!(
                        "{}",
                        serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()
                    ),
                    Err(e) => {
                        emit_err::<()>(meta, e);
                        std::process::exit(1);
                    }
                },
                Some(Commands::Diff { .. })
                | Some(Commands::Generate { .. })
                | Some(Commands::Schema { .. }) => unreachable!(),
            }
        }
    }
}

/// Discover schema changes. Never writes anything.
async fn run_diff<E: EntitySet>(
    entity_set: E,
    db: &sea_orm::DatabaseConnection,
    dangerous: bool,
    protected_table: &str,
) -> Result<DiffData, Box<dyn std::error::Error>> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let builder = entity_set
        .register(schema.builder())
        .exclude(protected_table);

    let change_set = builder.discover(db, dangerous).await?;
    let result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: backend,
            assumptions: true,
            allow_dangerous: dangerous,
        },
    );

    let statements: Vec<String> = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect();
    let schema_hash = fnv64_hex(statements.iter().map(String::as_str));
    let changes = summarize(
        &result
            .statements
            .iter()
            .map(|(_, s)| s.clone())
            .collect::<Vec<_>>(),
    );

    let warnings = result
        .warnings
        .iter()
        .map(|w| WarningJson {
            kind: format!("{:?}", w.kind),
            message: w.message.clone(),
        })
        .collect();

    let suggestions = result
        .suggestions
        .iter()
        .map(|s| SuggestionJson {
            kind: format!("{:?}", s.kind),
            message: s.message.clone(),
        })
        .collect();

    let unresolved = result
        .unresolved
        .iter()
        .map(|u| UnresolvedRenameJson {
            table: u.table.clone(),
            removed: u.removed.clone(),
            candidates: u.candidates.iter().map(|c| c.added.clone()).collect(),
        })
        .collect();

    Ok(DiffData {
        changes,
        statements,
        warnings,
        suggestions,
        unresolved,
        schema_hash,
    })
}

/// Build the entity-defined schema as SQL DDL without connecting to a database.
fn run_schema<E: EntitySet>(
    entity_set: E,
    database_backend: &str,
) -> Result<SchemaData, Box<dyn std::error::Error>> {
    let backend = match database_backend {
        "postgres" | "postgresql" => DbBackend::Postgres,
        "mysql" => DbBackend::MySql,
        "sqlite" => DbBackend::Sqlite,
        other => {
            return Err(format!(
                "Unknown database backend: {other}. Use postgres, mysql, or sqlite."
            )
            .into());
        }
    };
    let schema = Schema::new(backend);
    let builder = entity_set.register(schema.builder());
    let statements = builder
        .schema_statements()
        .into_iter()
        .map(|s| s.sql)
        .collect();
    Ok(SchemaData { statements })
}

/// Generate and write a migration file.
async fn run_generate<E: EntitySet>(
    entity_set: E,
    db: &sea_orm::DatabaseConnection,
    migration_dir: &str,
    name: &str,
    expected_schema_hash: &str,
    local_time: bool,
    dangerous: bool,
    renames: &[String],
    protected_table: &str,
) -> Result<GenerateData, Box<dyn std::error::Error>> {
    if name.contains('-') {
        return Err("`-` cannot be used in migration name".into());
    }

    let backend = db.get_database_backend();
    let schema = Schema::new(backend);
    let builder = entity_set
        .register(schema.builder())
        .exclude(protected_table);

    let change_set = builder.discover(db, dangerous).await?;
    let mut result = interpret_changes(
        change_set,
        &InterpretConfig {
            db_backend: backend,
            assumptions: true,
            allow_dangerous: dangerous,
        },
    );

    // Validate schema hash before proceeding
    let current_stmts: Vec<String> = result
        .statements
        .iter()
        .map(|(_, s)| s.sql.clone())
        .collect();
    let current_hash = fnv64_hex(current_stmts.iter().map(String::as_str));
    if current_hash != expected_schema_hash {
        return Err(format!(
            "Schema hash mismatch: expected {expected_schema_hash}, got {current_hash}. \
             Re-run `diff` to get a fresh schema hash."
        )
        .into());
    }

    // Error if there are still unresolved renames
    if !result.unresolved.is_empty() {
        // Try to apply provided --rename flags
        let cli_renames: Vec<(String, String, String)> =
            renames.iter().filter_map(|s| parse_rename_arg(s)).collect();

        let decisions = resolve_renames(&result.unresolved, &cli_renames)?;
        result.apply_rename_decisions(&decisions, backend);
    } else if !renames.is_empty() {
        // --rename flags provided but nothing to resolve — harmless, ignore
    }

    // After applying decisions, check if any unresolved remain
    if !result.unresolved.is_empty() {
        let remaining: Vec<String> = result
            .unresolved
            .iter()
            .map(|u| {
                let candidates = u
                    .candidates
                    .iter()
                    .map(|c| c.added.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}.{} -> [{}]", u.table, u.removed, candidates)
            })
            .collect();
        return Err(format!(
            "Unresolved ambiguous renames remain. Provide --rename for each:\n  {}",
            remaining.join("\n  ")
        )
        .into());
    }

    let stmts: Vec<_> = result.statements.into_iter().map(|(_, s)| s).collect();

    if stmts.is_empty() {
        return Err("No schema changes detected. Migration file not generated.".into());
    }

    let (timestamp, generated_at) = if local_time {
        let now = chrono::Local::now();
        (
            now.format("%Y%m%d_%H%M%S").to_string(),
            now.format("%Y-%m-%d %H:%M:%S %Z").to_string(),
        )
    } else {
        let now = Utc::now();
        (
            now.format("%Y%m%d_%H%M%S").to_string(),
            now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        )
    };

    let name_clean = name.trim().replace(' ', "_");
    let migration_name = format!("m{timestamp}_{name_clean}");
    let backend_name = match backend {
        DbBackend::MySql => "MySQL",
        DbBackend::Postgres => "PostgreSQL",
        DbBackend::Sqlite => "SQLite",
        _ => "Unknown",
    };

    let changes = summarize(&stmts);
    let meta = MigrationMetadata {
        version: VERSION,
        generated_at: &generated_at,
        backend: backend_name,
        changes: &changes,
    };

    let filepath = write_migration(migration_dir, &migration_name, &stmts, &meta)?;

    Ok(GenerateData {
        migration_name,
        filepath: filepath.display().to_string(),
        changes,
    })
}

/// Apply `--rename` CLI overrides to the unresolved list, returning decisions.
/// Errors if any ambiguity is left unresolved.
fn resolve_renames(
    unresolved: &[sea_orm::schema::resolver::AmbiguousRename],
    cli_renames: &[(String, String, String)],
) -> Result<Vec<sea_orm::schema::RenameDecision>, Box<dyn std::error::Error>> {
    use sea_orm::schema::RenameDecision;

    let mut decisions = Vec::new();
    let mut missing = Vec::new();

    for ambiguous in unresolved {
        if let Some((_, _, new_name)) = cli_renames
            .iter()
            .find(|(table, old, _)| *table == ambiguous.table && *old == ambiguous.removed)
        {
            if ambiguous.candidates.iter().any(|c| c.added == *new_name) {
                decisions.push(RenameDecision::Rename {
                    from: ambiguous.removed.clone(),
                    to: new_name.clone(),
                });
            } else {
                return Err(format!(
                    "--rename {}.{}:{} is invalid: '{}' is not among candidates [{}]",
                    ambiguous.table,
                    ambiguous.removed,
                    new_name,
                    new_name,
                    ambiguous
                        .candidates
                        .iter()
                        .map(|c| c.added.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .into());
            }
        } else {
            missing.push(format!(
                "{}.{} (candidates: {})",
                ambiguous.table,
                ambiguous.removed,
                ambiguous
                    .candidates
                    .iter()
                    .map(|c| c.added.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "Ambiguous renames require --rename flags:\n  {}",
            missing.join("\n  ")
        )
        .into());
    }

    Ok(decisions)
}

/// Parse a `--rename TABLE.OLD:NEW` string into `(table, old, new)`.
fn parse_rename_arg(s: &str) -> Option<(String, String, String)> {
    let (table_old, new) = s.split_once(':')?;
    let (table, old) = table_old.split_once('.')?;
    if table.is_empty() || old.is_empty() || new.is_empty() {
        return None;
    }
    Some((table.to_string(), old.to_string(), new.to_string()))
}

fn build_meta<M: MigratorTraitSelf>(migrator: &M, schema_hash: Option<String>) -> ApiMeta {
    ApiMeta {
        version: VERSION.to_string(),
        migrations_hash: Some(migrator.migrations_hash()),
        schema_hash,
    }
}

fn emit_err<T: serde::Serialize>(meta: ApiMeta, error: impl std::fmt::Display) {
    println!(
        "{}",
        serde_json::to_string(&ApiResponse::<T>::err(meta, error.to_string())).unwrap()
    );
}
