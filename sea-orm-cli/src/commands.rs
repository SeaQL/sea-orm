
use clap::ArgMatches;
use sea_orm_codegen::{EntityTransformer, OutputFile, WithSerde};
use std::{error::Error, fmt::Display, fs, io::Write, path::Path, process::Command, str::FromStr};
use url::Url;

pub async fn run_generate_command(matches: &ArgMatches<'_>) -> Result<(), Box<dyn Error>> {
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
            let singularize = args.is_present("SINGULARIZE");
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
            let url_host = url.host_str();

            let is_sqlite = url.scheme() == "sqlite";

            // Skip checking if it's SQLite
            if !is_sqlite {
                // Panic on any that are missing
                if url_username.is_empty() {
                    panic!("No username was found in the database url");
                }
                if url_host.is_none() {
                    panic!("No host was found in the database url");
                }
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

            let database_name = if !is_sqlite {
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

                database_name
            } else {
                Default::default()
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
                "sqlite" => {
                    use sea_schema::sqlite::SchemaDiscovery;
                    use sqlx::SqlitePool;

                    let connection = SqlitePool::connect(url.as_str()).await?;
                    let schema_discovery = SchemaDiscovery::new(connection);
                    let schema = schema_discovery.discover().await?;
                    schema
                        .tables
                        .into_iter()
                        .filter(|schema| filter_tables(&schema.name))
                        .filter(|schema| filter_hidden_tables(&schema.name))
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

pub fn run_migrate_command(matches: &ArgMatches<'_>) -> Result<(), Box<dyn Error>> {
    let migrate_subcommand = matches.subcommand();
    // If it's `migrate init`
    if let ("init", Some(args)) = migrate_subcommand {
        let migration_dir = args.value_of("MIGRATION_DIR").unwrap();
        let migration_dir = match migration_dir.ends_with('/') {
            true => migration_dir.to_string(),
            false => format!("{}/", migration_dir),
        };
        println!("Initializing migration directory...");
        macro_rules! write_file {
            ($filename: literal) => {
                write_file!($filename, $filename);
            };
            ($filename: literal, $template: literal) => {
                let filepath = [&migration_dir, $filename].join("");
                println!("Creating file `{}`", filepath);
                let path = Path::new(&filepath);
                let prefix = path.parent().unwrap();
                fs::create_dir_all(prefix).unwrap();
                let mut file = fs::File::create(path)?;
                let content = include_str!(concat!("../template/migration/", $template));
                file.write_all(content.as_bytes())?;
            };
        }
        write_file!("src/lib.rs");
        write_file!("src/m20220101_000001_create_table.rs");
        write_file!("src/main.rs");
        write_file!("Cargo.toml", "_Cargo.toml");
        write_file!("README.md");
        println!("Done!");
        // Early exit!
        return Ok(());
    }
    let (subcommand, migration_dir, steps, verbose) = match migrate_subcommand {
        // Catch all command with pattern `migrate xxx`
        (subcommand, Some(args)) => {
            let migration_dir = args.value_of("MIGRATION_DIR").unwrap();
            let steps = args.value_of("NUM_MIGRATION");
            let verbose = args.is_present("VERBOSE");
            (subcommand, migration_dir, steps, verbose)
        }
        // Catch command `migrate`, this will be treated as `migrate up`
        _ => {
            let migration_dir = matches.value_of("MIGRATION_DIR").unwrap();
            let verbose = matches.is_present("VERBOSE");
            ("up", migration_dir, None, verbose)
        }
    };
    // Construct the `--manifest-path`
    let manifest_path = if migration_dir.ends_with('/') {
        format!("{}Cargo.toml", migration_dir)
    } else {
        format!("{}/Cargo.toml", migration_dir)
    };
    // Construct the arguments that will be supplied to `cargo` command
    let mut args = vec![
        "run",
        "--manifest-path",
        manifest_path.as_str(),
        "--",
        subcommand,
    ];
    if let Some(steps) = steps {
        args.extend(["-n", steps]);
    }
    if verbose {
        args.push("-v");
    }
    // Run migrator CLI on user's behalf
    println!("Running `cargo {}`", args.join(" "));
    Command::new("cargo").args(args).spawn()?.wait()?;
    Ok(())
}

pub fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{}", error);
    ::std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::AppSettings;
    use crate::cli;

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: RelativeUrlWithoutBase"
    )]
    fn test_generate_entity_no_protocol() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "://root:root@localhost:3306/database",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap())).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "There is no database name as part of the url path: postgresql://root:root@localhost:3306"
    )]
    fn test_generate_entity_no_database_section() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "postgresql://root:root@localhost:3306",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap())).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "There is no database name as part of the url path: mysql://root:root@localhost:3306/"
    )]
    fn test_generate_entity_no_database_path() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "mysql://root:root@localhost:3306/",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap())).unwrap();
    }

    #[test]
    #[should_panic(expected = "No username was found in the database url")]
    fn test_generate_entity_no_username() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "mysql://:root@localhost:3306/database",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap())).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: PoolTimedOut")]
    fn test_generate_entity_no_password() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "mysql://root:@localhost:3306/database",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap())).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: EmptyHost")]
    fn test_generate_entity_no_host() {
        let matches = cli::build_cli()
            .setting(AppSettings::NoBinaryName)
            .get_matches_from(vec![
                "generate",
                "entity",
                "--database-url",
                "postgres://root:root@/database",
            ]);

        smol::block_on(run_generate_command(matches.subcommand().1.unwrap())).unwrap();
    }
}
