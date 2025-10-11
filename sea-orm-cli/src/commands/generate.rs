use core::time;
use sea_orm_codegen::{
    DateTimeCrate as CodegenDateTimeCrate, EntityTransformer, EntityWriterContext, OutputFile,
    WithPrelude, WithSerde,
};
use std::{error::Error, fs, io::Write, path::Path, process::Command, str::FromStr};
use tracing_subscriber::{EnvFilter, prelude::*};
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
            frontend_format,
            include_hidden_tables,
            tables,
            ignore_tables,
            max_connections,
            acquire_timeout,
            output_dir,
            database_schema,
            database_url,
            with_prelude,
            with_serde,
            serde_skip_deserializing_primary_key,
            serde_skip_hidden_column,
            with_copy_enums,
            date_time_crate,
            lib,
            model_extra_derives,
            model_extra_attributes,
            enum_extra_derives,
            enum_extra_attributes,
            column_extra_derives,
            seaography,
            impl_active_model_behavior,
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

            let _database_name = if !is_sqlite {
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
                    #[cfg(not(feature = "sqlx-mysql"))]
                    {
                        panic!("mysql feature is off")
                    }
                    #[cfg(feature = "sqlx-mysql")]
                    {
                        use sea_schema::mysql::discovery::SchemaDiscovery;
                        use sqlx::MySql;

                        println!("Connecting to MySQL ...");
                        let connection = sqlx_connect::<MySql>(
                            max_connections,
                            acquire_timeout,
                            url.as_str(),
                            None,
                        )
                        .await?;
                        println!("Discovering schema ...");
                        let schema_discovery = SchemaDiscovery::new(connection, _database_name);
                        let schema = schema_discovery.discover().await?;
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
                }
                "sqlite" => {
                    #[cfg(not(feature = "sqlx-sqlite"))]
                    {
                        panic!("sqlite feature is off")
                    }
                    #[cfg(feature = "sqlx-sqlite")]
                    {
                        use sea_schema::sqlite::discovery::SchemaDiscovery;
                        use sqlx::Sqlite;

                        println!("Connecting to SQLite ...");
                        let connection = sqlx_connect::<Sqlite>(
                            max_connections,
                            acquire_timeout,
                            url.as_str(),
                            None,
                        )
                        .await?;
                        println!("Discovering schema ...");
                        let schema_discovery = SchemaDiscovery::new(connection);
                        let schema = schema_discovery
                            .discover()
                            .await?
                            .merge_indexes_into_table();
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
                }
                "postgres" | "postgresql" => {
                    #[cfg(not(feature = "sqlx-postgres"))]
                    {
                        panic!("postgres feature is off")
                    }
                    #[cfg(feature = "sqlx-postgres")]
                    {
                        use sea_schema::postgres::discovery::SchemaDiscovery;
                        use sqlx::{Postgres, Row};
                        use std::collections::{HashMap, HashSet};

                        println!("Connecting to Postgres ...");
                        let schema = database_schema.as_deref().unwrap_or("public");
                        let pool = sqlx_connect::<Postgres>(
                            max_connections,
                            acquire_timeout,
                            url.as_str(),
                            Some(schema),
                        )
                        .await?;

                        // Discover all schemas that need to be included based on cross-schema references
                        println!("Analyzing cross-schema dependencies ...");

                        let mut schemas_to_discover = HashSet::new();
                        schemas_to_discover.insert(schema.to_string());

                        // Query to find all schemas referenced by the target schema via foreign keys
                        let fk_query = r#"
                            SELECT DISTINCT
                                fn.nspname AS foreign_schema
                            FROM pg_constraint c
                            JOIN pg_class t ON c.conrelid = t.oid
                            JOIN pg_namespace n ON t.relnamespace = n.oid
                            JOIN pg_class ft ON c.confrelid = ft.oid
                            JOIN pg_namespace fn ON ft.relnamespace = fn.oid
                            WHERE c.contype = 'f'
                                AND n.nspname = $1
                                AND fn.nspname != $1
                        "#;

                        let fk_rows = sqlx::query(fk_query).bind(schema).fetch_all(&pool).await?;

                        for row in fk_rows {
                            let foreign_schema: String = row.get("foreign_schema");
                            schemas_to_discover.insert(foreign_schema);
                        }

                        // Query to find all schemas that have enums used by the target schema
                        let enum_schema_query = r#"
                            SELECT DISTINCT tn.nspname AS type_schema
                            FROM pg_attribute a
                            JOIN pg_class c ON a.attrelid = c.oid
                            JOIN pg_namespace n ON c.relnamespace = n.oid
                            JOIN pg_type t ON a.atttypid = t.oid
                            JOIN pg_namespace tn ON t.typnamespace = tn.oid
                            WHERE n.nspname = $1
                                AND t.typtype = 'e'
                                AND tn.nspname != $1
                        "#;

                        let enum_schema_rows = sqlx::query(enum_schema_query)
                            .bind(schema)
                            .fetch_all(&pool)
                            .await?;

                        for row in enum_schema_rows {
                            let type_schema: String = row.get("type_schema");
                            schemas_to_discover.insert(type_schema);
                        }

                        println!("Will discover schemas: {:?}", schemas_to_discover);

                        // Discover all enums from all relevant schemas
                        let enum_query = r#"
                            SELECT n.nspname as schema, t.typname as typename, e.enumlabel
                            FROM pg_type t
                            JOIN pg_enum e ON t.oid = e.enumtypid
                            JOIN pg_namespace n ON n.oid = t.typnamespace
                            ORDER BY schema, typename, e.enumsortorder
                        "#;

                        let enum_rows = sqlx::query(enum_query).fetch_all(&pool).await?;
                        let mut all_enums: HashMap<String, Vec<String>> = HashMap::new();
                        for row in enum_rows {
                            let typename: String = row.get("typename");
                            let enumlabel: String = row.get("enumlabel");
                            all_enums
                                .entry(typename)
                                .or_insert_with(Vec::new)
                                .push(enumlabel);
                        }

                        // Discover tables from all relevant schemas
                        let mut all_tables = Vec::new();

                        for discover_schema in schemas_to_discover.iter() {
                            println!("Discovering tables in schema: {}", discover_schema);
                            let discovery = SchemaDiscovery::new(pool.clone(), discover_schema);
                            let discovered = discovery.discover().await?;
                            all_tables.extend(discovered.tables);
                        }

                        println!(
                            "Discovered {} tables across {} schemas",
                            all_tables.len(),
                            schemas_to_discover.len()
                        );

                        let table_stmts = all_tables
                            .into_iter()
                            .filter(|schema| filter_tables(&schema.info.name))
                            .filter(|schema| filter_hidden_tables(&schema.info.name))
                            .filter(|schema| filter_skip_tables(&schema.info.name))
                            .map(|schema| schema.write())
                            .collect();
                        (database_schema, table_stmts)
                    }
                }
                _ => unimplemented!("{} is not supported", url.scheme()),
            };
            println!("... discovered.");

            let writer_context = EntityWriterContext::new(
                expanded_format,
                frontend_format,
                WithPrelude::from_str(&with_prelude).expect("Invalid prelude option"),
                WithSerde::from_str(&with_serde).expect("Invalid serde derive option"),
                with_copy_enums,
                date_time_crate.into(),
                schema_name,
                lib,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
                enum_extra_derives,
                enum_extra_attributes,
                column_extra_derives,
                seaography,
                impl_active_model_behavior,
            );
            let output = EntityTransformer::transform(table_stmts)?.generate(&writer_context);

            let dir = Path::new(&output_dir);
            fs::create_dir_all(dir)?;

            for OutputFile { name, content } in output.files.iter() {
                let file_path = dir.join(name);
                println!("Writing {}", file_path.display());
                let mut file = fs::File::create(file_path)?;
                file.write_all(content.as_bytes())?;
            }

            // Format each of the files
            for OutputFile { name, .. } in output.files.iter() {
                let exit_status = Command::new("rustfmt").arg(dir.join(name)).status()?; // Get the status code
                if !exit_status.success() {
                    // Propagate the error if any
                    return Err(format!("Fail to format file `{name}`").into());
                }
            }

            println!("... Done.");
        }
    }

    Ok(())
}

async fn sqlx_connect<DB>(
    max_connections: u32,
    acquire_timeout: u64,
    url: &str,
    schema: Option<&str>,
) -> Result<sqlx::Pool<DB>, Box<dyn Error>>
where
    DB: sqlx::Database,
    for<'a> &'a mut <DB as sqlx::Database>::Connection: sqlx::Executor<'a>,
{
    let mut pool_options = sqlx::pool::PoolOptions::<DB>::new()
        .max_connections(max_connections)
        .acquire_timeout(time::Duration::from_secs(acquire_timeout));
    // Set search_path for Postgres to allow cross-schema type resolution
    // MySQL & SQLite connection initialize with schema `None`
    if let Some(schema) = schema {
        // Always include "$user" and public in search_path to support cross-schema type references
        // This allows types (like enums) defined in any schema to be discovered
        // PostgreSQL will search in order: target schema, then "$user", then public
        // See: https://www.postgresql.org/docs/current/ddl-schemas.html#DDL-SCHEMAS-PATH
        let sql = format!("SET search_path = '{schema}', \"$user\", public");
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
    use clap::Parser;

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
