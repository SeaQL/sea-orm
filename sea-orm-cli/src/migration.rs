use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    let mut app = App::new("sea-schema-migration")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name("VERBOSE")
                .long("verbose")
                .short("v")
                .help("Show debug messages")
                .takes_value(false)
                .global(true),
        );
    for subcommand in get_subcommands() {
        app = app.subcommand(subcommand);
    }
    app
}

pub fn get_subcommands() -> Vec<App<'static, 'static>> {
    vec![
        SubCommand::with_name("fresh")
            .about("Drop all tables from the database, then reapply all migrations"),
        SubCommand::with_name("refresh")
            .about("Rollback all applied migrations, then reapply all migrations"),
        SubCommand::with_name("reset").about("Rollback all applied migrations"),
        SubCommand::with_name("status").about("Check the status of all migrations"),
        SubCommand::with_name("up")
            .about("Apply pending migrations")
            .arg(
                Arg::with_name("NUM_MIGRATION")
                    .long("num")
                    .short("n")
                    .help("Number of pending migrations to be applied")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("VERSION_MIGRATION")
                    .long("version")
                    .short("V")
                    .help("Version of pending migrations to be applied")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("FORCE_MIGRATION")
                    .long("force")
                    .short("f")
                    .help("force version of pending migrations to be applied")
                    .takes_value(false),
            ),
            SubCommand::with_name("down")
            .about("Rollback applied migrations")
            .arg(
                Arg::with_name("NUM_MIGRATION")
                    .long("num")
                    .short("n")
                    .help("Number of pending migrations to be rolled back")
                    .takes_value(true)
                    .default_value("1"),
            )
            .arg(
                Arg::with_name("VERSION_MIGRATION")
                    .long("version")
                    .short("V")
                    .help("Version of pending migrations to be rolled back")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("FORCE_MIGRATION")
                    .long("force")
                    .short("f")
                    .help("force version of pending migrations to be rolled back")
                    .takes_value(false),
            ),
    ]
}
