use clap::Parser;
use dotenvy::dotenv;
use std::{error::Error, fmt::Display, process::exit};
use tracing_subscriber::{prelude::*, EnvFilter};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_cli::{run_migrate_generate, run_migrate_init, MigrateSubcommands};

use super::MigratorTrait;

const MIGRATION_DIR: &str = "./";

pub async fn run_cli<M>(migrator: M)
where
    M: MigratorTrait,
{
    dotenv().ok();
    let cli = Cli::parse();

    let db = db_connect(&cli.database_url, &cli.database_schema).await;
    let db_ref = match db {
        Ok(ref d) => Ok(d),
        Err(e) => Err(e),
    };

    run_migrate(
        migrator,
        &cli.migration_dir,
        db_ref,
        cli.command,
        cli.verbose,
    )
    .await
    .unwrap_or_else(handle_error);
}

pub async fn run_migrate<M>(
    _: M,
    migration_dir: &str,
    db: Result<&DatabaseConnection, String>,
    command: Option<MigrateSubcommands>,
    verbose: bool,
) -> Result<(), Box<dyn Error>>
where
    M: MigratorTrait,
{
    let filter = match verbose {
        true => "debug",
        false => "sea_orm_migration=info",
    };

    let filter_layer = EnvFilter::try_new(filter).unwrap();

    if verbose {
        let fmt_layer = tracing_subscriber::fmt::layer();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init()
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_level(false)
            .without_time();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init()
    };

    match command {
        Some(MigrateSubcommands::Fresh) => M::fresh(db?).await?,
        Some(MigrateSubcommands::Refresh) => M::refresh(db?).await?,
        Some(MigrateSubcommands::Reset) => M::reset(db?).await?,
        Some(MigrateSubcommands::Status) => M::status(db?).await?,
        Some(MigrateSubcommands::Up { num }) => M::up(db?, num).await?,
        Some(MigrateSubcommands::Down { num }) => M::down(db?, Some(num)).await?,
        Some(MigrateSubcommands::Init) => run_migrate_init(migration_dir)?,
        Some(MigrateSubcommands::Generate {
            migration_name,
            universal_time: _,
            local_time,
        }) => run_migrate_generate(migration_dir, &migration_name, !local_time)?,
        _ => M::up(db?, None).await?,
    };

    Ok(())
}

async fn db_connect(
    db_url: &Option<String>,
    schema: &Option<String>,
) -> Result<DatabaseConnection, String> {
    let url = match db_url {
        Some(url) => url,
        None => return Err("Environment variable 'DATABASE_URL' not set".to_owned()),
    };
    let schema = schema.clone().unwrap_or_else(|| "public".to_string());

    let connect_options = ConnectOptions::new(url)
        .set_schema_search_path(schema)
        .to_owned();

    Database::connect(connect_options)
        .await
        .map_err(|e| format!("Fail to acquire database connection: {}", e))
}

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    #[arg(short = 'v', long, global = true, help = "Show debug messages")]
    verbose: bool,

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
        short = 'd',
        long,
        default_value = MIGRATION_DIR,
        env = "MIGRATION_DIR",
        help = "Migration directory"
    )]
    migration_dir: String,

    #[command(subcommand)]
    command: Option<MigrateSubcommands>,
}

fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{error}");
    exit(1);
}
