#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(
    missing_debug_implementations,
    clippy::print_stderr,
    clippy::print_stdout
)]

//! # SeaConnection
//!
//! Database Connection Layer for SeaQL

pub mod database;
pub mod driver;
pub mod error;
pub mod executor;
pub mod metric;
pub mod util;

pub use database::*;
pub use driver::*;
pub use error::*;
pub use executor::*;

#[cfg(feature = "sqlx-dep")]
pub use sqlx;

pub use sea_query;
