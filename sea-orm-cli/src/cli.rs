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
                        .short("url")
                        .help("Database URL")
                        .takes_value(true)
                        .required(true)
                        .env("DATABASE_URL"),
                )
                .arg(
                    Arg::with_name("DATABASE_SCHEMA")
                        .long("database-schema")
                        .short("schema")
                        .help("Database schema")
                        .takes_value(true)
                        .required(true)
                        .env("DATABASE_SCHEMA"),
                )
                .arg(
                    Arg::with_name("OUTPUT_DIR")
                        .long("output_dir")
                        .short("o")
                        .help("Entity file output directory")
                        .takes_value(true)
                        .default_value("./"),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp);

    App::new("sea-orm")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(entity_subcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
}
