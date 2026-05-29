use chrono::Utc;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use sea_orm::{ConnectOptions, Database, DbBackend, Schema};
use sea_orm_migration::MigratorTraitSelf;
use tracing_subscriber::{EnvFilter, prelude::*};

use crate::filter::filter_protected_drops;
use crate::{EntitySet, codegen::MigrationMetadata, fs::write_migration, summary::summarize};

const VERSION: &str = env!("CARGO_PKG_VERSION");

//TODO: Move this to cli later
#[derive(Parser)]
#[command(
    name = "entity",
    about = "Entity-first migration tool for SeaORM",
    version
)]
struct Cli {
    #[arg(short = 'v', long, global = true, help = "Show debug messages")]
    verbose: bool,

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

//TODO: Move this to cli later
#[derive(Subcommand)]
enum Commands {
    /// Generate a migration file from entity definitions by diffing against the database.
    ///
    /// By default uses the live database provided via --database-url or env.
    /// In the future, an --ephemeral flag will instead spin up an in-memory database, apply existing
    /// migrations, and discover changes without a live connection.
    Generate {
        /// Path to the migration crate directory
        #[arg(long, default_value = "../migration")]
        migration_dir: String,

        /// Name for the migration (like `add_users`)
        #[arg(required = true, help = "Name of the new migration")]
        name: String,

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

        /// Allow dangerous operations (e.g. dropping tables)
        #[arg(long, default_value_t = true)]
        allow_dangerous: bool,
        // Future: --ephemeral flag will be added here for no-live-db gen
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
            help = "Number of applied migrations to be rolled back",
            display_order = 90
        )]
        num: u32,
    },
}

/// Run the entity CLI with the given entity set and migrator
///
/// Call this from your entity crate's `main.rs`:
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() {
///     sea_orm_entity::cli::run_cli(Entities, migration::Migrator).await;
/// }
/// ```
pub async fn run_cli<E, M>(entity_set: E, migrator: M)
where
    E: EntitySet,
    M: MigratorTraitSelf,
{
    dotenv().ok();
    let cli = Cli::parse();

    let url = cli
        .database_url
        .expect("Environment variable 'DATABASE_URL' not set");
    let schema = cli.database_schema.unwrap_or_else(|| "public".to_owned());
    let verbose = cli.verbose;

    match cli.command {
        Some(Commands::Generate {
            migration_dir,
            name,
            local_time,
            universal_time: _,
            allow_dangerous,
        }) => {
            // Extract the migration tracker table name so we never generate DROP statements for it.
            let migration_table = migrator.migration_table_name().to_string();

            println!("Connecting to database...");
            let db = Database::connect(
                ConnectOptions::new(url)
                    .set_schema_search_path(schema)
                    .to_owned(),
            )
            .await
            .expect("Failed to connect to database");

            // Future: when --ephemeral is added, build an in-memory db here instead,
            // apply existing migrations via the migrator, then pass it to generate_from_db.

            if let Err(e) = generate_from_db(
                entity_set,
                db,
                &migration_dir,
                &name,
                local_time,
                allow_dangerous,
                &migration_table,
            )
            .await
            {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }

        migrate_cmd => {
            // All migration execution logic lives in MigratorTraitSelf (sea-orm-migration)
            init_tracing(verbose);
            let db = Database::connect(
                ConnectOptions::new(url)
                    .set_schema_search_path(schema)
                    .to_owned(),
            )
            .await
            .expect("Failed to connect to database");

            let result = match migrate_cmd {
                Some(Commands::Up { num }) => migrator.up(&db, num).await,
                Some(Commands::Down { num }) => migrator.down(&db, Some(num)).await,
                Some(Commands::Fresh) => migrator.fresh(&db).await,
                Some(Commands::Refresh) => migrator.refresh(&db).await,
                Some(Commands::Reset) => migrator.reset(&db).await,
                Some(Commands::Status) => migrator.status(&db).await,
                // No subcommand: apply all pending migrations
                None => migrator.up(&db, None).await,
                Some(Commands::Generate { .. }) => unreachable!(),
            };

            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// Core generation logic
async fn generate_from_db<E: EntitySet>(
    entity_set: E,
    db: sea_orm::DatabaseConnection,
    migration_dir: &str,
    name: &str,
    local_time: bool,
    dangerous: bool,
    protected_table: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if name.contains('-') {
        return Err("`-` cannot be used in migration name".into());
    }

    let backend = db.get_database_backend();

    let schema = Schema::new(backend);
    let builder = entity_set.register(schema.builder());

    println!("Discovering schema changes...");
    let raw = builder.discover(&db, dangerous).await?;

    // Never drop migration tracker table
    let stmts = filter_protected_drops(raw, protected_table);

    if stmts.is_empty() {
        println!("No schema changes detected. Migration file not generated");
        return Ok(());
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
    println!("Generated migration: {}", filepath.display());
    println!("Changes ({}):", changes.len());
    for change in &changes {
        println!("  - {change}");
    }

    Ok(())
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        "debug"
    } else {
        "sea_orm_migration=info"
    };
    let filter_layer = EnvFilter::try_new(filter).unwrap();
    let fmt_layer = tracing_subscriber::fmt::layer();
    if verbose {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(
                fmt_layer
                    .with_target(false)
                    .with_level(false)
                    .without_time(),
            )
            .init();
    }
}
