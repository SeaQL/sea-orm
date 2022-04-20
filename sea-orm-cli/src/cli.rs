use clap::{App, AppSettings, Arg, SubCommand};
use sea_schema::get_cli_subcommands;

pub fn build_cli() -> App<'static, 'static> {
    let entity_subcommand = SubCommand::with_name("generate")
        .about("Codegen related commands")
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("entity")
                .about("Generate entity")
                .arg(
                    Arg::with_name("DATABASE_URL")
                        .long("database-url")
                        .short("u")
                        .help("Database URL")
                        .takes_value(true)
                        .required(true)
                        .env("DATABASE_URL"),
                )
                .arg(
                    Arg::with_name("DATABASE_SCHEMA")
                        .long("database-schema")
                        .short("s")
                        .help("Database schema")
                        .long_help("Database schema\n \
                        - For MySQL, this argument is ignored.\n \
                        - For PostgreSQL, this argument is optional with default value 'public'.")
                        .takes_value(true)
                        .env("DATABASE_SCHEMA"),
                )
                .arg(
                    Arg::with_name("OUTPUT_DIR")
                        .long("output-dir")
                        .short("o")
                        .help("Entity file output directory")
                        .takes_value(true)
                        .default_value("./"),
                )
                .arg(
                    Arg::with_name("INCLUDE_HIDDEN_TABLES")
                        .long("include-hidden-tables")
                        .help("Generate entity file for hidden tables (i.e. table name starts with an underscore)")
                        .takes_value(false),
                )
                .arg(
                    Arg::with_name("TABLES")
                        .long("tables")
                        .short("t")
                        .use_delimiter(true)
                        .help("Generate entity file for specified tables only (comma seperated)")
                        .takes_value(true)
                        .conflicts_with("INCLUDE_HIDDEN_TABLES"),
                )
                .arg(
                    Arg::with_name("EXPANDED_FORMAT")
                        .long("expanded-format")
                        .help("Generate entity file of expanded format")
                        .takes_value(false)
                        .conflicts_with("COMPACT_FORMAT"),
                )
                .arg(
                    Arg::with_name("COMPACT_FORMAT")
                        .long("compact-format")
                        .help("Generate entity file of compact format")
                        .takes_value(false)
                        .conflicts_with("EXPANDED_FORMAT"),
                )
                .arg(
                    Arg::with_name("WITH_SERDE")
                        .long("with-serde")
                        .help("Automatically derive serde Serialize / Deserialize traits for the entity (none, serialize, deserialize, both)")
                        .takes_value(true)
                        .default_value("none")
                )
                .arg(
                    Arg::with_name("MAX_CONNECTIONS")
                        .long("max-connections")
                        .help("The maximum amount of connections to use when connecting to the database.")
                        .takes_value(true)
                        .default_value("1")
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp);

    let arg_migration_dir = Arg::with_name("MIGRATION_DIR")
        .long("migration-dir")
        .short("d")
        .help("Migration script directory")
        .takes_value(true)
        .default_value("./migration");
    let mut migrate_subcommands = SubCommand::with_name("migrate")
        .about("Migration related commands")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize migration directory")
                .arg(arg_migration_dir.clone()),
        )
        .arg(arg_migration_dir.clone());
    for subcommand in get_cli_subcommands!() {
        migrate_subcommands =
            migrate_subcommands.subcommand(subcommand.arg(arg_migration_dir.clone()));
    }

    App::new("sea-orm-cli")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(entity_subcommand)
        .subcommand(migrate_subcommands)
        .arg(
            Arg::with_name("VERBOSE")
                .long("verbose")
                .short("v")
                .help("Show debug messages")
                .takes_value(false)
                .global(true),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
}
