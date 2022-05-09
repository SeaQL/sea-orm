use chrono::Local;
use clap::ArgMatches;
use regex::Regex;
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
            if args.is_present("VERBOSE") {
                let _ = tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::DEBUG)
                    .with_test_writer()
                    .try_init();
            }

            let max_connections = args
                .value_of("MAX_CONNECTIONS")
                .map(str::parse::<u32>)
                .transpose()?
                .unwrap();

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
                    use sqlx::MySql;

                    let connection = connect::<MySql>(max_connections, url.as_str()).await?;
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
                    use sea_schema::sqlite::discovery::SchemaDiscovery;
                    use sqlx::Sqlite;

                    let connection = connect::<Sqlite>(max_connections, url.as_str()).await?;
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
                    use sqlx::Postgres;

                    let schema = args.value_of("DATABASE_SCHEMA").unwrap_or("public");
                    let connection = connect::<Postgres>(max_connections, url.as_str()).await?;
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

async fn connect<DB>(max_connections: u32, url: &str) -> Result<sqlx::Pool<DB>, Box<dyn Error>>
where
    DB: sqlx::Database,
{
    sqlx::pool::PoolOptions::<DB>::new()
        .max_connections(max_connections)
        .connect(url)
        .await
        .map_err(Into::into)
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
    } else if let ("generate", Some(args)) = migrate_subcommand {
        let migration_dir = args.value_of("MIGRATION_DIR").unwrap();
        let migration_name = args.value_of("MIGRATION_NAME").unwrap();
        println!("Generating new migration...");

        // build new migration filename
        let now = Local::now();
        let migration_name = format!(
            "m{}_{}",
            now.format("%Y%m%d_%H%M%S").to_string(),
            migration_name
        );

        create_new_migration(&migration_name, migration_dir)?;
        update_migrator(&migration_name, migration_dir)?;
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

fn create_new_migration(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migration_filepath = Path::new(migration_dir)
        .join("src")
        .join(format!("{}.rs", &migration_name));
    println!("Creating migration file `{}`", migration_filepath.display());
    // TODO: make OS agnostic
    let migration_template =
        include_str!("../template/migration/src/m20220101_000001_create_table.rs");
    let migration_content =
        migration_template.replace("m20220101_000001_create_table", &migration_name);
    let mut migration_file = fs::File::create(migration_filepath)?;
    migration_file.write_all(migration_content.as_bytes())?;
    Ok(())
}

fn update_migrator(migration_name: &str, migration_dir: &str) -> Result<(), Box<dyn Error>> {
    let migrator_filepath = Path::new(migration_dir).join("src").join("lib.rs");
    println!(
        "Adding migration `{}` to `{}`",
        migration_name,
        migrator_filepath.display()
    );
    let migrator_content = fs::read_to_string(&migrator_filepath)?;
    let mut updated_migrator_content = migrator_content.clone();

    // create a backup of the migrator file in case something goes wrong
    let migrator_backup_filepath = migrator_filepath.clone().with_file_name("lib.rs.bak");
    fs::copy(&migrator_filepath, &migrator_backup_filepath)?;
    let mut migrator_file = fs::File::create(&migrator_filepath)?;

    // find existing mod declarations, add new line
    let mod_regex = Regex::new(r"mod\s+(?P<name>m\d{8}_\d{6}_\w+);")?;
    let mods: Vec<_> = mod_regex.captures_iter(&migrator_content).collect();
    let mods_end = mods.last().unwrap().get(0).unwrap().end() + 1;
    updated_migrator_content.insert_str(mods_end, format!("mod {};\n", migration_name).as_str());

    // build new vector from declared migration modules
    let mut migrations: Vec<&str> = mods
        .iter()
        .map(|cap| cap.name("name").unwrap().as_str())
        .collect();
    migrations.push(migration_name);
    let mut boxed_migrations = migrations
        .iter()
        .map(|migration| format!("            Box::new({}::Migration),", migration))
        .collect::<Vec<String>>()
        .join("\n");
    boxed_migrations.push_str("\n");
    let boxed_migrations = format!("vec![\n{}        ]\n", boxed_migrations);
    let vec_regex = Regex::new(r"vec!\[[\s\S]+\]\n")?;
    let updated_migrator_content = vec_regex.replace(&updated_migrator_content, &boxed_migrations);

    migrator_file.write_all(updated_migrator_content.as_bytes())?;
    fs::remove_file(&migrator_backup_filepath)?;
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
    use crate::cli;
    use clap::AppSettings;

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
    #[test]
    fn test_create_new_migration() {
        let migration_name = "test_name";
        let migration_dir = "/tmp/sea_orm_cli_test_new_migration/";
        fs::create_dir_all(format!("{}src", migration_dir)).unwrap();
        create_new_migration(migration_name, migration_dir).unwrap();
        let migration_filepath = Path::new(migration_dir)
            .join("src")
            .join(format!("{}.rs", migration_name));
        assert!(migration_filepath.exists());
        let migration_content = fs::read_to_string(migration_filepath).unwrap();
        let migration_content =
            migration_content.replace(&migration_name, "m20220101_000001_create_table");
        assert_eq!(
            &migration_content,
            include_str!("../template/migration/src/m20220101_000001_create_table.rs")
        );
        fs::remove_dir_all("/tmp/sea_orm_cli_test_new_migration/").unwrap();
    }

    #[test]
    fn test_update_migrator() {
        let migration_name = "test_name";
        let migration_dir = "/tmp/sea_orm_cli_test_update_migrator/";
        fs::create_dir_all(format!("{}src", migration_dir)).unwrap();
        let migrator_filepath = Path::new(migration_dir).join("src").join("lib.rs");
        fs::copy("./template/migration/src/lib.rs", &migrator_filepath).unwrap();
        update_migrator(migration_name, migration_dir).unwrap();
        assert!(&migrator_filepath.exists());
        let migrator_content = fs::read_to_string(&migrator_filepath).unwrap();
        let mod_regex = Regex::new(r"mod (?P<name>\w+);").unwrap();
        let migrations: Vec<&str> = mod_regex
            .captures_iter(&migrator_content)
            .map(|cap| cap.name("name").unwrap().as_str())
            .collect();
        assert_eq!(migrations.len(), 2);
        assert_eq!(
            *migrations.first().unwrap(),
            "m20220101_000001_create_table"
        );
        assert_eq!(migrations.last().unwrap(), &migration_name);
        let boxed_regex = Regex::new(r"Box::new\((?P<name>\S+)::Migration\)").unwrap();
        let migrations: Vec<&str> = boxed_regex
            .captures_iter(&migrator_content)
            .map(|cap| cap.name("name").unwrap().as_str())
            .collect();
        assert_eq!(migrations.len(), 2);
        assert_eq!(
            *migrations.first().unwrap(),
            "m20220101_000001_create_table"
        );
        assert_eq!(migrations.last().unwrap(), &migration_name);
        fs::remove_dir_all("/tmp/sea_orm_cli_test_update_migrator/").unwrap();
    }
}
