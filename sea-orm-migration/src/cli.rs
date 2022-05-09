use clap::App;
use dotenv::dotenv;
use std::{fmt::Display, process::exit};
use tracing_subscriber::{prelude::*, EnvFilter};

use sea_orm::{Database, DbConn};
use sea_orm_cli::build_cli;

use super::MigratorTrait;

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

pub async fn get_matches<M>(_: M, db: &DbConn, app: App<'static, 'static>)
where
    M: MigratorTrait,
{
    let matches = app.get_matches();
    let mut verbose = false;
    let filter = match matches.subcommand() {
        (_, None) => "sea_schema::migration=info",
        (_, Some(args)) => match args.is_present("VERBOSE") {
            true => {
                verbose = true;
                "debug"
            }
            false => "sea_schema::migration=info",
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

fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{}", error);
    exit(1);
}
