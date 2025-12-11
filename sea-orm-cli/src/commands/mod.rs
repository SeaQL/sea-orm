use std::fmt::Display;

#[cfg(feature = "cli")]
pub mod config;

#[cfg(feature = "codegen")]
pub mod generate;
pub mod migrate;

#[cfg(feature = "cli")]
pub use config::*;

#[cfg(feature = "codegen")]
pub use generate::*;
pub use migrate::*;

pub fn handle_error<E>(error: E)
where
    E: Display,
{
    eprintln!("{error}");
    ::std::process::exit(1);
}
