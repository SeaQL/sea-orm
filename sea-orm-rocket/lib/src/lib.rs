//! SeaORM Rocket support crate.
#![deny(missing_docs)]

/// Re-export of the `figment` crate.
#[doc(inline)]
pub use rocket::figment;

pub use rocket;

mod config;
mod database;
mod error;
mod pool;

pub use self::config::Config;
pub use self::database::{Connection, Database, Initializer};
pub use self::error::Error;
pub use self::pool::{MockPool, Pool};

pub use sea_orm_rocket_codegen::*;
