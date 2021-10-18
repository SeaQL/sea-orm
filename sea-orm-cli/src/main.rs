use clap::ArgMatches;
use dotenv::dotenv;
use log::LevelFilter;
use sea_orm_codegen::{EntityTransformer, OutputFile, WithSerde};
use std::{error::Error, fmt::Display, fs, io::Write, path::Path, process::Command, str::FromStr};
use url::Url;

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
            // The database should be a valid URL that can be parsed
            // protocol://username:password@host/database_name
            let url = match Url::parse(
                args.value_of("DATABASE_URL")
                    .expect("No database url could be found"),
            ) {
                Ok(url) => url,
                Err(e) => panic!("Invalid database url: {}", e),
            };

            let output_dir = args.value_of("OUTPUT_DIR").unwrap();
            let include_hidden_tables = args.is_present("INCLUDE_HIDDEN_TABLES");
            let tables = args
                .values_of("TABLES")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            let expanded_format = args.is_present("EXPANDED_FORMAT");
            let with_serde = args.value_of("WITH_SERDE").unwrap();
            if args.is_present("VERBOSE") {
                let _ = ::env_logger::builder()
                    .filter_level(LevelFilter::Debug)
                    .is_test(true)
                    .try_init();
            }

            // The database name should be the first element of the path string
            //
            // Throwing an error if there is no database name since it might be
            // accepted by the database without it, while we're looking to dump
            // information from a particular database
            let database_name = url
                .path_segments()
                .expect(&format!(
                    "There is no database name as part of the url path: {}",
                    url.as_str()
                ))
                .next()
                .unwrap();

            // An empty string as the database name is also an error
            if database_name.is_empty() {
                panic!(
                    "There is no database name as part of the url path: {}",
                    url.as_str()
                );
            }

            // Closures for filtering tables
            let filter_tables = |table: &str| -> bool {
                if !tables.is_empty() {
                    return tables.contains(&table);
                }

                true
            };
            let filter_hidden_tables = |table: &str| -> bool {
                if include_hidden_tables {
                    true
                } else {
                    !table.starts_with('_')
                }
            };

            let table_stmts = match url.scheme() {
                "mysql" => {
                    use sea_schema::mysql::discovery::SchemaDiscovery;
                    use sqlx::MySqlPool;

                    let connection = MySqlPool::connect(url.as_str()).await?;
                    let schema_discovery = SchemaDiscovery::new(connection, database_name);
                    let schema = schema_discovery.discover().await;
                    schema
                        .tables
                        .into_iter()
                        .filter(|schema| filter_tables(&schema.info.name))
                        .filter(|schema| filter_hidden_tables(&schema.info.name))
                        .map(|schema| schema.write())
                        .collect()
                }
                "postgres" | "postgresql" => {
                    use sea_schema::postgres::discovery::SchemaDiscovery;
                    use sqlx::PgPool;

                    let schema = args.value_of("DATABASE_SCHEMA").unwrap_or("public");
                    let connection = PgPool::connect(url.as_str()).await?;
                    let schema_discovery = SchemaDiscovery::new(connection, schema);
                    let schema = schema_discovery.discover().await;
                    schema
                        .tables
                        .into_iter()
                        .filter(|schema| filter_tables(&schema.info.name))
                        .filter(|schema| filter_hidden_tables(&schema.info.name))
                        .map(|schema| schema.write())
                        .collect()
                }
                _ => unimplemented!("{} is not supported", url.scheme()),
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

            // Format each of the files
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
