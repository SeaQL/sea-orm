use clap::ArgMatches;
use dotenv::dotenv;
use log::LevelFilter;
use sea_orm_codegen::{EntityTransformer, OutputFile, WithSerde};
use std::{error::Error, fmt::Display, fs, io::Write, path::Path, process::Command, str::FromStr};

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
            let url = args.value_of("DATABASE_URL").unwrap();
            let output_dir = args.value_of("OUTPUT_DIR").unwrap();
            let include_hidden_tables = args.is_present("INCLUDE_HIDDEN_TABLES");
            let tables = args
                .values_of("TABLES")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            let expanded_format = args.is_present("EXPANDED_FORMAT");
            let with_serde = args.value_of("WITH_SERDE").unwrap();
            let filter_tables = |table: &str| -> bool {
                if tables.len() > 0 {
                    return tables.contains(&table);
                }

                true
            };
            let filter_hidden_tables = |table: &str| -> bool {
                if include_hidden_tables {
                    true
                } else {
                    !table.starts_with("_")
                }
            };
            if args.is_present("VERBOSE") {
                let _ = ::env_logger::builder()
                    .filter_level(LevelFilter::Debug)
                    .is_test(true)
                    .try_init();
            }

            let table_stmts = if url.starts_with("mysql://") {
                use sea_schema::mysql::discovery::SchemaDiscovery;
                use sqlx::MySqlPool;

                let url_parts: Vec<&str> = url.split("/").collect();
                let schema = url_parts.last().unwrap();
                let connection = MySqlPool::connect(url).await?;
                let schema_discovery = SchemaDiscovery::new(connection, schema);
                let schema = schema_discovery.discover().await;
                schema
                    .tables
                    .into_iter()
                    .filter(|schema| filter_tables(&schema.info.name))
                    .filter(|schema| filter_hidden_tables(&schema.info.name))
                    .map(|schema| schema.write())
                    .collect()
            } else if url.starts_with("postgres://") || url.starts_with("postgresql://") {
                use sea_schema::postgres::discovery::SchemaDiscovery;
                use sqlx::PgPool;

                let schema = args.value_of("DATABASE_SCHEMA").unwrap_or("public");
                let connection = PgPool::connect(url).await?;
                let schema_discovery = SchemaDiscovery::new(connection, schema);
                let schema = schema_discovery.discover().await;
                schema
                    .tables
                    .into_iter()
                    .filter(|schema| filter_tables(&schema.info.name))
                    .filter(|schema| filter_hidden_tables(&schema.info.name))
                    .map(|schema| schema.write())
                    .collect()
            } else {
                panic!("This database is not supported ({})", url)
            };

            let output = EntityTransformer::transform(table_stmts)?
                .generate(expanded_format, WithSerde::from_str(with_serde).unwrap());

            let dir = Path::new(output_dir);
            fs::create_dir_all(dir)?;

            for OutputFile { name, content } in output.files.iter() {
                let file_path = dir.join(name);
                let mut file = fs::File::create(file_path)?;
                file.write_all(content.as_bytes())?;
            }
            for OutputFile { name, .. } in output.files.iter() {
                Command::new("rustfmt")
                    .arg(dir.join(name))
                    .spawn()?
                    .wait()?;
            }
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
