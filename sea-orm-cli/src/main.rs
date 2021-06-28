use clap::ArgMatches;
use dotenv::dotenv;
use sea_orm_codegen::EntityGenerator;
use std::{error::Error, fmt::Display};

mod cli;

#[async_std::main]
async fn main() {
    dotenv().ok();

    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
        ("generate", Some(matches)) => run_generate_command(matches)
            .await
            .unwrap_or_else(handle_error),
        _ => unreachable!("You should never see this message"),
    }
}

async fn run_generate_command(matches: &ArgMatches<'_>) -> Result<(), Box<dyn Error>> {
    match matches.subcommand() {
        ("entity", Some(args)) => {
            let uri = args.value_of("DATABASE_URI").unwrap();
            let schema = args.value_of("DATABASE_SCHEMA").unwrap();
            let output_dir = args.value_of("OUTPUT_DIR").unwrap();
            EntityGenerator::discover(uri, schema)
                .await?
                .transform()?
                .generate(output_dir)?;
        }
        _ => unreachable!("You should never see this message"),
    };

    Ok(())
}

fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{}", error);
    ::std::process::exit(1);
}
