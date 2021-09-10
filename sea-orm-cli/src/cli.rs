use clap::{App, AppSettings, Arg, SubCommand};

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
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp);

    App::new("sea-orm")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(entity_subcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
}
