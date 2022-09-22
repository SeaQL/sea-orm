use clap::{ArgEnum, ArgGroup, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Cli {
    #[clap(action, global = true, short, long, help = "Show debug messages")]
    pub verbose: bool,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum Commands {
    #[clap(about = "Codegen related commands")]
    #[clap(arg_required_else_help = true)]
    Generate {
        #[clap(subcommand)]
        command: GenerateSubcommands,
    },
    #[clap(about = "Migration related commands")]
    Migrate {
        #[clap(
            value_parser,
            global = true,
            short = 'd',
            long,
            help = "Migration script directory.
If your migrations are in their own crate,
you can provide the root of that crate.
If your migrations are in a submodule of your app,
you should provide the directory of that submodule.",
            default_value = "./migration"
        )]
        migration_dir: String,

        #[clap(
            value_parser,
            global = true,
            short = 's',
            long,
            env = "DATABASE_SCHEMA",
            long_help = "Database schema\n \
                        - For MySQL, this argument is ignored.\n \
                        - For PostgreSQL, this argument is optional with default value 'public'."
        )]
        database_schema: Option<String>,

        #[clap(
            value_parser,
            global = true,
            short = 'u',
            long,
            env = "DATABASE_URL",
            help = "Database URL"
        )]
        database_url: Option<String>,

        #[clap(subcommand)]
        command: Option<MigrateSubcommands>,
    },
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum MigrateSubcommands {
    #[clap(about = "Initialize migration directory")]
    Init,
    #[clap(about = "Generate a new, empty migration")]
    Generate {
        #[clap(
            value_parser,
            required = true,
            takes_value = true,
            help = "Name of the new migration"
        )]
        migration_name: String,
    },
    #[clap(about = "Drop all tables from the database, then reapply all migrations")]
    Fresh,
    #[clap(about = "Rollback all applied migrations, then reapply all migrations")]
    Refresh,
    #[clap(about = "Rollback all applied migrations")]
    Reset,
    #[clap(about = "Check the status of all migrations")]
    Status,
    #[clap(about = "Apply pending migrations")]
    Up {
        #[clap(
            value_parser,
            short,
            long,
            default_value = "1",
            help = "Number of pending migrations to apply"
        )]
        num: u32,
    },
    #[clap(value_parser, about = "Rollback applied migrations")]
    Down {
        #[clap(
            value_parser,
            short,
            long,
            default_value = "1",
            help = "Number of applied migrations to be rolled back"
        )]
        num: u32,
    },
}

#[derive(Subcommand, PartialEq, Eq, Debug)]
pub enum GenerateSubcommands {
    #[clap(about = "Generate entity")]
    #[clap(arg_required_else_help = true)]
    #[clap(group(ArgGroup::new("formats").args(&["compact-format", "expanded-format"])))]
    #[clap(group(ArgGroup::new("group-tables").args(&["tables", "include-hidden-tables"])))]
    Entity {
        #[clap(action, long, help = "Generate entity file of compact format")]
        compact_format: bool,

        #[clap(action, long, help = "Generate entity file of expanded format")]
        expanded_format: bool,

        #[clap(
            action,
            long,
            help = "Generate entity file for hidden tables (i.e. table name starts with an underscore)"
        )]
        include_hidden_tables: bool,

        #[clap(
            value_parser,
            short = 't',
            long,
            use_value_delimiter = true,
            takes_value = true,
            help = "Generate entity file for specified tables only (comma separated)"
        )]
        tables: Option<String>,

        #[clap(
            value_parser,
            long,
            use_value_delimiter = true,
            takes_value = true,
            default_value = "seaql_migrations",
            help = "Skip generating entity file for specified tables (comma separated)"
        )]
        ignore_tables: Vec<String>,

        #[clap(
            value_parser,
            long,
            default_value = "1",
            help = "The maximum amount of connections to use when connecting to the database."
        )]
        max_connections: u32,

        #[clap(
            value_parser,
            short = 'o',
            long,
            default_value = "./",
            help = "Entity file output directory"
        )]
        output_dir: String,

        #[clap(
            value_parser,
            short = 's',
            long,
            env = "DATABASE_SCHEMA",
            default_value = "public",
            long_help = "Database schema\n \
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
            arg_enum,
            value_parser,
            long,
            default_value = "chrono",
            help = "The datetime crate to use for generating entities."
        )]
        date_time_crate: DateTimeCrate,
    },
}

#[derive(ArgEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateTimeCrate {
    Chrono,
    Time,
}
