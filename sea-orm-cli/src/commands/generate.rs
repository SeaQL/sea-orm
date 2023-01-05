use sea_orm_codegen::{
    DateTimeCrate as CodegenDateTimeCrate, EntityTransformer, EntityWriterContext, OutputFile,
    WithSerde,
};
use std::{error::Error, fs, io::Write, path::Path, process::Command, str::FromStr};
use tracing_subscriber::{prelude::*, EnvFilter};
use url::Url;

use crate::{DateTimeCrate, GenerateSubcommands};

pub async fn run_generate_command(
    command: GenerateSubcommands,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    match command {
        GenerateSubcommands::Entity {
            compact_format: _,
            expanded_format,
            include_hidden_tables,
            tables,
            ignore_tables,
            max_connections,
            output_dir,
            database_schema,
            database_url,
            with_serde,
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
            with_copy_enums,
            date_time_crate,
            lib,
            model_extra_derives,
            model_extra_attributes,
        } => {
            if verbose {
                let _ = tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::DEBUG)
                    .with_test_writer()
                    .try_init();
            } else {
                let filter_layer = EnvFilter::try_new("sea_orm_codegen=info").unwrap();
                let fmt_layer = tracing_subscriber::fmt::layer()
                    .with_target(false)
                    .with_level(false)
                    .without_time();

                let _ = tracing_subscriber::registry()
                    .with(filter_layer)
                    .with(fmt_layer)
                    .try_init();
            }

            // The database should be a valid URL that can be parsed
            // protocol://username:password@host/database_name
            let url = Url::parse(&database_url)?;

            // Make sure we have all the required url components
            //
            // Missing scheme will have been caught by the Url::parse() call
            // above
            let is_sqlite = url.scheme() == "sqlite";

            // Closures for filtering tables
            let filter_tables =
                |table: &String| -> bool { tables.is_empty() || tables.contains(table) };

            let filter_hidden_tables = |table: &str| -> bool {
                if include_hidden_tables {
                    true
                } else {
                    !table.starts_with('_')
                }
            };

            let filter_skip_tables = |table: &String| -> bool { !ignore_tables.contains(table) };

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

            let (schema_name, table_stmts) = match url.scheme() {
                "mysql" => {
                    use sea_schema::mysql::discovery::SchemaDiscovery;
                    use sqlx::MySql;

                    let connection = connect::<MySql>(max_connections, url.as_str(), None).await?;
                    let schema_discovery = SchemaDiscovery::new(connection, database_name);
                    let schema = schema_discovery.discover().await;
                    let table_stmts = schema
                        .tables
                        .into_iter()
                        .filter(|schema| filter_tables(&schema.info.name))
                        .filter(|schema| filter_hidden_tables(&schema.info.name))
                        .filter(|schema| filter_skip_tables(&schema.info.name))
                        .map(|schema| schema.write())
                        .collect();
                    (None, table_stmts)
                }
                "sqlite" => {
                    use sea_schema::sqlite::discovery::SchemaDiscovery;
                    use sqlx::Sqlite;

                    let connection = connect::<Sqlite>(max_connections, url.as_str(), None).await?;
                    let schema_discovery = SchemaDiscovery::new(connection);
                    let schema = schema_discovery.discover().await?;
                    let table_stmts = schema
                        .tables
                        .into_iter()
                        .filter(|schema| filter_tables(&schema.name))
                        .filter(|schema| filter_hidden_tables(&schema.name))
                        .filter(|schema| filter_skip_tables(&schema.name))
                        .map(|schema| schema.write())
                        .collect();
                    (None, table_stmts)
                }
                "postgres" | "postgresql" => {
                    use sea_schema::postgres::discovery::SchemaDiscovery;
                    use sqlx::Postgres;

                    let schema = &database_schema;
                    let connection =
                        connect::<Postgres>(max_connections, url.as_str(), Some(schema)).await?;
                    let schema_discovery = SchemaDiscovery::new(connection, schema);
                    let schema = schema_discovery.discover().await;
                    let table_stmts = schema
                        .tables
                        .into_iter()
                        .filter(|schema| filter_tables(&schema.info.name))
                        .filter(|schema| filter_hidden_tables(&schema.info.name))
                        .filter(|schema| filter_skip_tables(&schema.info.name))
                        .map(|schema| schema.write())
                        .collect();
                    (Some(schema.schema), table_stmts)
                }
                _ => unimplemented!("{} is not supported", url.scheme()),
            };

            let writer_context = EntityWriterContext::new(
                expanded_format,
                WithSerde::from_str(&with_serde).expect("Invalid serde derive option"),
                with_copy_enums,
                date_time_crate.into(),
                schema_name,
                lib,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
            );
            let output = EntityTransformer::transform(table_stmts)?.generate(&writer_context);

            let dir = Path::new(&output_dir);
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
    }

    Ok(())
}

async fn connect<DB>(
    max_connections: u32,
    url: &str,
    schema: Option<&str>,
) -> Result<sqlx::Pool<DB>, Box<dyn Error>>
where
    DB: sqlx::Database,
    for<'a> &'a mut <DB as sqlx::Database>::Connection: sqlx::Executor<'a>,
{
    let mut pool_options = sqlx::pool::PoolOptions::<DB>::new().max_connections(max_connections);
    // Set search_path for Postgres, E.g. Some("public") by default
    // MySQL & SQLite connection initialize with schema `None`
    if let Some(schema) = schema {
        let sql = format!("SET search_path = '{}'", schema);
        pool_options = pool_options.after_connect(move |conn, _| {
            let sql = sql.clone();
            Box::pin(async move {
                sqlx::Executor::execute(conn, sql.as_str())
                    .await
                    .map(|_| ())
            })
        });
    }
    pool_options.connect(url).await.map_err(Into::into)
}

impl From<DateTimeCrate> for CodegenDateTimeCrate {
    fn from(date_time_crate: DateTimeCrate) -> CodegenDateTimeCrate {
        match date_time_crate {
            DateTimeCrate::Chrono => CodegenDateTimeCrate::Chrono,
            DateTimeCrate::Time => CodegenDateTimeCrate::Time,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::StructOpt;

    use super::*;
    use crate::{Cli, Commands};

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: RelativeUrlWithoutBase"
    )]
    fn test_generate_entity_no_protocol() {
        let cli = Cli::parse_from([
            "sea-orm-cli",
            "generate",
            "entity",
            "--database-url",
            "://root:root@localhost:3306/database",
        ]);

        match cli.command {
            Commands::Generate { command } => {
                smol::block_on(run_generate_command(command, cli.verbose)).unwrap();
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[should_panic(
        expected = "There is no database name as part of the url path: postgresql://root:root@localhost:3306"
    )]
    fn test_generate_entity_no_database_section() {
        let cli = Cli::parse_from([
            "sea-orm-cli",
            "generate",
            "entity",
            "--database-url",
            "postgresql://root:root@localhost:3306",
        ]);

        match cli.command {
            Commands::Generate { command } => {
                smol::block_on(run_generate_command(command, cli.verbose)).unwrap();
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[should_panic(
        expected = "There is no database name as part of the url path: mysql://root:root@localhost:3306/"
    )]
    fn test_generate_entity_no_database_path() {
        let cli = Cli::parse_from([
            "sea-orm-cli",
            "generate",
            "entity",
            "--database-url",
            "mysql://root:root@localhost:3306/",
        ]);

        match cli.command {
            Commands::Generate { command } => {
                smol::block_on(run_generate_command(command, cli.verbose)).unwrap();
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: PoolTimedOut")]
    fn test_generate_entity_no_password() {
        let cli = Cli::parse_from([
            "sea-orm-cli",
            "generate",
            "entity",
            "--database-url",
            "mysql://root:@localhost:3306/database",
        ]);

        match cli.command {
            Commands::Generate { command } => {
                smol::block_on(run_generate_command(command, cli.verbose)).unwrap();
            }
            _ => unreachable!(),
        }
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: EmptyHost")]
    fn test_generate_entity_no_host() {
        let cli = Cli::parse_from([
            "sea-orm-cli",
            "generate",
            "entity",
            "--database-url",
            "postgres://root:root@/database",
        ]);

        match cli.command {
            Commands::Generate { command } => {
                smol::block_on(run_generate_command(command, cli.verbose)).unwrap();
            }
            _ => unreachable!(),
        }
    }
}
