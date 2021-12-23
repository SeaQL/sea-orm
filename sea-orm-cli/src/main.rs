use clap::ArgMatches;
use dotenv::dotenv;
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
            let output_dir = args.value_of("OUTPUT_DIR").unwrap();
            let include_hidden_tables = args.is_present("INCLUDE_HIDDEN_TABLES");
            let tables = args
                .values_of("TABLES")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            let expanded_format = args.is_present("EXPANDED_FORMAT");
            let with_serde = args.value_of("WITH_SERDE").unwrap();
            if args.is_present("VERBOSE") {
                let _ = tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::DEBUG)
                    .with_test_writer()
                    .try_init();
            }

            // The database should be a valid URL that can be parsed
            // protocol://username:password@host/database_name
            let url = Url::parse(
                args.value_of("DATABASE_URL")
                    .expect("No database url could be found"),
            )?;

            // Make sure we have all the required url components
            //
            // Missing scheme will have been caught by the Url::parse() call
            // above
            let url_username = url.username();
            let url_password = url.password();
            let url_host = url.host_str();

            // Panic on any that are missing
            if url_username.is_empty() {
                panic!("No username was found in the database url");
            }
            if url_password.is_none() {
                panic!("No password was found in the database url");
            }
            if url_host.is_none() {
                panic!("No host was found in the database url");
            }

            // The database name should be the first element of the path string
            //
            // Throwing an error if there is no database name since it might be
            // accepted by the database without it, while we're looking to dump
            // information from a particular database
            let database_name = url
                .path_segments()
                .unwrap_or_else(|| {
                    panic!(
                        "There is no database name as part of the url path: {}",
                        url.as_str()
                    )
                })
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

#[cfg(test)]
mod tests {
    use clap::AppSettings;
    use url::ParseError;

    use super::*;

    #[async_std::test]
    async fn test_generate_entity_no_protocol() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "://root:root@localhost:3306/database",
            ]);

        let result = std::panic::catch_unwind(|| {
            smol::block_on(run_generate_command(matches.subcommand().1.unwrap()))
        });

        // Make sure result is a ParseError
        match result {
            Ok(Err(e)) => match e.downcast::<ParseError>() {
                Ok(_) => (),
                Err(e) => panic!("Expected ParseError but got: {:?}", e),
            },
            _ => panic!("Should have panicked"),
        }
    }

    #[test]
    #[should_panic]
    fn test_generate_entity_no_database_section() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "postgresql://root:root@localhost:3306",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap()))
            .unwrap_or_else(handle_error);
    }

    #[test]
    #[should_panic]
    fn test_generate_entity_no_database_path() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "mysql://root:root@localhost:3306/",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap()))
            .unwrap_or_else(handle_error);
    }

    #[test]
    #[should_panic]
    fn test_generate_entity_no_username() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "mysql://:root@localhost:3306/database",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap()))
            .unwrap_or_else(handle_error);
    }

    #[test]
    #[should_panic]
    fn test_generate_entity_no_password() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "mysql://root:@localhost:3306/database",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap()))
            .unwrap_or_else(handle_error);
    }

    #[async_std::test]
    async fn test_generate_entity_no_host() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "postgres://root:root@/database",
            ]);

        let result = std::panic::catch_unwind(|| {
            smol::block_on(run_generate_command(matches.subcommand().1.unwrap()))
        });

        // Make sure result is a ParseError
        match result {
            Ok(Err(e)) => match e.downcast::<ParseError>() {
                Ok(_) => (),
                Err(e) => panic!("Expected ParseError but got: {:?}", e),
            },
            _ => panic!("Should have panicked"),
        }
    }
}
