use clap::{ArgEnum, ArgGroup, Parser, Subcommand};

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
                        - For PostgreSQL, this argument is optional with default value 'public'."
        )]
        database_schema: String,

        #[clap(
            value_parser,
            short = 'u',
            long,
            env = "DATABASE_URL",
            help = "Database URL"
        )]
        database_url: String,

        #[clap(
            value_parser,
            long,
            default_value = "none",
            help = "Automatically derive serde Serialize / Deserialize traits for the entity (none,\
                serialize, deserialize, both)"
        )]
        with_serde: String,

        #[clap(
            action,
            long,
            default_value = "false",
            long_help = "Automatically derive the Copy trait on generated enums.\n\
            Enums generated from a database don't have associated data by default, and as such can\
            derive Copy.
            "
        )]
        with_copy_enums: bool,

        #[clap(
            long,
            default_value = "true",
            help = "Generate module names in singular."
        )]
        singularize: bool,

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
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate a new, empty migration")
                .arg(
                    Arg::with_name("MIGRATION_NAME")
                        .help("Name of the new migation")
                        .required(true)
                        .takes_value(true),
                )
                .arg(arg_migration_dir.clone()),
        )
        .arg(arg_migration_dir.clone());
    for subcommand in get_subcommands() {
        migrate_subcommands =
            migrate_subcommands.subcommand(subcommand.arg(arg_migration_dir.clone()));
    }

#[derive(ArgEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateTimeCrate {
    Chrono,
    Time,
}
