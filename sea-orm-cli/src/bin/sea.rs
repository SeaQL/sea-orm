//! COPY FROM bin/main.rs

use clap::StructOpt;
use dotenv::dotenv;
use sea_orm_cli::{handle_error, run_generate_command, run_migrate_command, Cli, Commands};

#[async_std::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();
    let verbose = cli.verbose;

    match cli.command {
        Commands::Generate { command } => {
            run_generate_command(command, verbose)
                .await
                .unwrap_or_else(handle_error);
        }
        Commands::Migrate {
            database_schema,
            migration_dir,
            command,
        } => run_migrate_command(
            command,
            database_schema.as_str(),
            migration_dir.as_str(),
            verbose,
        )
        .unwrap_or_else(handle_error),
    }
}
