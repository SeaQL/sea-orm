use std::future::Future;

use clap::Parser;
use dotenvy::dotenv;
use std::process::exit;

use sea_orm::{ConnectOptions, Database, DbConn, DbErr};
use sea_orm_cli::{MigrateSubcommands, run_migrate_generate, run_migrate_init};

use crate::response::{ApiMeta, ApiResponse};
use super::MigratorTraitSelf;

const MIGRATION_DIR: &str = "./";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn run_cli<M>(migrator: M)
where
    M: MigratorTraitSelf,
{
    run_cli_with_connection(migrator, Database::connect).await;
}

/// Same as [`run_cli`] where you provide the function to create the [`DbConn`].
pub async fn run_cli_with_connection<M, F, Fut>(migrator: M, make_connection: F)
where
    M: MigratorTraitSelf,
    F: FnOnce(ConnectOptions) -> Fut,
    Fut: Future<Output = Result<DbConn, DbErr>>,
{
    dotenv().ok();
    let cli = Cli::parse();

    let url = cli
        .database_url
        .expect("Environment variable 'DATABASE_URL' not set");
    let schema = cli.database_schema.unwrap_or_else(|| "public".to_owned());

    let connect_options = ConnectOptions::new(url)
        .set_schema_search_path(schema)
        .to_owned();

    let db = match make_connection(connect_options).await {
        Ok(db) => db,
        Err(e) => {
            let meta = migrator_meta(&migrator);
            emit_err::<()>(meta, e);
            exit(1);
        }
    };

    run_migrate(migrator, &db, cli.command).await;
}

pub async fn run_migrate<M>(migrator: M, db: &DbConn, command: Option<MigrateSubcommands>)
where
    M: MigratorTraitSelf,
{
    let meta = migrator_meta(&migrator);

    match command {
        Some(MigrateSubcommands::Status) => {
            match migrator.status(db).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Up { num }) => {
            match migrator.up(db, num).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Down { num }) => {
            match migrator.down(db, Some(num)).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Fresh) => {
            match migrator.fresh(db).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Refresh) => {
            match migrator.refresh(db).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Reset) => {
            match migrator.reset(db).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Init) => {
            match run_migrate_init(MIGRATION_DIR) {
                Ok(()) => {
                    #[derive(serde::Serialize)]
                    struct InitData { migration_dir: &'static str }
                    println!("{}", serde_json::to_string(&ApiResponse::ok(meta, InitData { migration_dir: MIGRATION_DIR })).unwrap());
                }
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        Some(MigrateSubcommands::Generate { migration_name, universal_time: _, local_time }) => {
            match run_migrate_generate(MIGRATION_DIR, &migration_name, !local_time) {
                Ok(()) => {
                    #[derive(serde::Serialize)]
                    struct GenData<'a> { migration_name: &'a str, migration_dir: &'static str }
                    println!("{}", serde_json::to_string(&ApiResponse::ok(meta, GenData { migration_name: &migration_name, migration_dir: MIGRATION_DIR })).unwrap());
                }
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
        // No subcommand: apply all pending migrations
        None => {
            match migrator.up(db, None).await {
                Ok(data) => println!("{}", serde_json::to_string(&ApiResponse::ok(meta, data)).unwrap()),
                Err(e) => { emit_err::<()>(meta, e); exit(1); }
            }
        }
    }
}

fn migrator_meta<M: MigratorTraitSelf>(migrator: &M) -> ApiMeta {
    ApiMeta {
        version: VERSION.to_string(),
        migrations_hash: Some(migrator.migrations_hash()),
        schema_hash: None,
    }
}

fn emit_err<T: serde::Serialize>(meta: ApiMeta, error: impl std::fmt::Display) {
    println!("{}", serde_json::to_string(&ApiResponse::<T>::err(meta, error.to_string())).unwrap());
}

#[derive(Parser)]
#[command(version)]
pub struct Cli {
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

    #[command(subcommand)]
    command: Option<MigrateSubcommands>,
}
