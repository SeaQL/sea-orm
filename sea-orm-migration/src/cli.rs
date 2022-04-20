//! Run migrator CLI

use crate::MigratorTrait;
use clap::{App, AppSettings, Arg};
use dotenv::dotenv;
use sea_orm::{Database, DbConn};
use sea_schema::get_cli_subcommands;
use std::{fmt::Display, process::exit};
use tracing_subscriber::{prelude::*, EnvFilter};

#[allow(dead_code)]
/// Migrator CLI application
pub async fn run_cli<M>(migrator: M)
where
    M: MigratorTrait,
{
    dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");
    let db = &Database::connect(&url).await.unwrap();
    let app = build_cli();
    get_matches(migrator, db, app).await;
}

async fn get_matches<M>(_: M, db: &DbConn, app: App<'static, 'static>)
where
    M: MigratorTrait,
{
    let matches = app.get_matches();
    let mut verbose = false;
    let filter = match matches.subcommand() {
        (_, None) => "sea_orm::migration=info",
        (_, Some(args)) => match args.is_present("VERBOSE") {
            true => {
                verbose = true;
                "debug"
            }
            false => "sea_orm::migration=info",
        },
    };
    let filter_layer = EnvFilter::try_new(filter).unwrap();
    if verbose {
        let fmt_layer = tracing_subscriber::fmt::layer();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init()
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_level(false)
            .without_time();
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init()
    };
    match matches.subcommand() {
        ("fresh", _) => M::fresh(db).await,
        ("refresh", _) => M::refresh(db).await,
        ("reset", _) => M::reset(db).await,
        ("status", _) => M::status(db).await,
        ("up", None) => M::up(db, None).await,
        ("down", None) => M::down(db, Some(1)).await,
        ("up", Some(args)) => {
            let str = args.value_of("NUM_MIGRATION").unwrap_or_default();
            let steps = str.parse().ok();
            M::up(db, steps).await
        }
        ("down", Some(args)) => {
            let str = args.value_of("NUM_MIGRATION").unwrap();
            let steps = str.parse().ok().unwrap_or(1);
            M::down(db, Some(steps)).await
        }
        _ => M::up(db, None).await,
    }
    .unwrap_or_else(handle_error);
}

fn build_cli() -> App<'static, 'static> {
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
    for subcommand in get_cli_subcommands!() {
        app = app.subcommand(subcommand);
    }
    app
}

fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{}", error);
    exit(1);
}
