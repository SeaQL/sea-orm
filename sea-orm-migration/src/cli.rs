use clap::Parser;
use dotenv::dotenv;
use std::{error::Error, fmt::Display, process::exit};
use tracing_subscriber::{prelude::*, EnvFilter};

use sea_orm::{Database, DbConn, ConnectOptions};
use sea_orm_cli::{run_migrate_generate, run_migrate_init, MigrateSubcommands};

use super::MigratorTrait;

const MIGRATION_DIR: &str = "./";

pub async fn run_cli<M>(migrator: M)
where
    M: MigratorTrait,
{
    dotenv().ok();
    let cli = Cli::parse();

    let url = cli.database_url.expect("Environment variable 'DATABASE_URL' not set");
    let schema = cli.database_schema.unwrap_or("public".to_owned());

    let connect_options = ConnectOptions::new(url)
        .set_schema_search_path(schema)
        .to_owned();
    let db = &Database::connect(connect_options).await.expect("Fail to acquire database connection");

    run_migrate(migrator, db, cli.command, cli.verbose)
        .await
        .unwrap_or_else(handle_error);
}

pub async fn run_migrate<M>(
    _: M,
    db: &DbConn,
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
        Some(MigrateSubcommands::Fresh) => M::fresh(db).await?,
        Some(MigrateSubcommands::Refresh) => M::refresh(db).await?,
        Some(MigrateSubcommands::Reset) => M::reset(db).await?,
        Some(MigrateSubcommands::Status) => M::status(db).await?,
        Some(MigrateSubcommands::Up { num }) => M::up(db, Some(num)).await?,
        Some(MigrateSubcommands::Down { num }) => M::down(db, Some(num)).await?,
        Some(MigrateSubcommands::Init) => run_migrate_init(MIGRATION_DIR)?,
        Some(MigrateSubcommands::Generate { migration_name }) => {
            run_migrate_generate(MIGRATION_DIR, &migration_name)?
        }
        _ => M::up(db, None).await?,
    };

    Ok(())
}

#[derive(Parser)]
#[clap(version)]
pub struct Cli {
    #[clap(action, short = 'v', long, global = true, help = "Show debug messages")]
    verbose: bool,

    #[clap(
        value_parser,
        global = true,
        short = 's',
        long,
        env = "DATABASE_SCHEMA",
        long_help = "Database schema\n \
                    - For MySQL, this argument is ignored.\n \
                    - For PostgreSQL, this argument is optional with default value 'public'."
    )]
    database_schema: Option<String>,

    #[clap(
        value_parser,
        global = true,
        short = 'u',
        long,
        env = "DATABASE_URL",
        help = "Database URL"
    )]
    database_url: Option<String>,

    #[clap(subcommand)]
    command: Option<MigrateSubcommands>,
}

fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{}", error);
    exit(1);
}
